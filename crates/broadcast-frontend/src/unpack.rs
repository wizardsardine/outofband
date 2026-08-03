//! In-browser archive extraction: bytes in, labeled text items out.
//!
//! Pure by design — no DOM access, no network, no filesystem, no global
//! state — because this is the one place a hostile input is processed
//! (PLAN.md section 5 and 7). A dropped file affects only the tab of the
//! person who dropped it, and even there extraction streams against a
//! fixed budget, caps entry counts, and refuses nested archives.

use core::fmt;
use std::io::{self, Cursor, Read};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use tx_core::Decoded;

/// Streamed decompression budget shared across one file load operation.
pub(crate) const DECOMPRESSION_BUDGET_BYTES: u64 = 256 * 1024 * 1024;
/// Maximum number of archive entries (files and directories alike) one file
/// load operation may contain.
const MAX_ENTRIES: usize = 1000;
/// Read chunk size, so the budget is checked incrementally rather than
/// after decompressing an entry in full.
const CHUNK_BYTES: usize = 64 * 1024;

/// One expanded item, ready to be handed to `queue::analyze` alongside a
/// label describing where it came from.
#[derive(Debug, Clone, PartialEq)]
pub struct UnpackedItem {
    /// Sanitized entry name (or the original filename for a plain file) —
    /// safe to display as-is.
    pub label: String,
    pub text: String,
}

/// Everything that can abort extraction. Every variant renders a message
/// fit for direct display; none of them come from a panic.
#[derive(Debug)]
pub enum UnpackError {
    /// The selected archives contain more than [`MAX_ENTRIES`] entries.
    TooManyEntries,
    /// Streamed decompression exceeded [`DECOMPRESSION_BUDGET_BYTES`].
    BudgetExceeded,
    /// An entry is itself a zip, gzip or tar archive.
    NestedArchive,
    Zip(zip::result::ZipError),
    Io(io::Error),
}

impl fmt::Display for UnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnpackError::TooManyEntries => {
                write!(f, "selected archives have more than {MAX_ENTRIES} entries")
            }
            UnpackError::BudgetExceeded => write!(
                f,
                "total decompressed size exceeds the {} MiB budget",
                DECOMPRESSION_BUDGET_BYTES / (1024 * 1024)
            ),
            UnpackError::NestedArchive => write!(f, "nested archives are not allowed"),
            UnpackError::Zip(e) => write!(f, "malformed zip archive: {e}"),
            UnpackError::Io(e) => write!(f, "malformed archive: {e}"),
        }
    }
}

impl std::error::Error for UnpackError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            UnpackError::TooManyEntries
            | UnpackError::BudgetExceeded
            | UnpackError::NestedArchive => None,
            UnpackError::Zip(e) => Some(e),
            UnpackError::Io(e) => Some(e),
        }
    }
}

/// The three recognised container formats, detected by magic bytes only —
/// never by filename or extension. Anything else is a plain file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Container {
    Zip,
    Gzip,
    Tar,
    Plain,
}

fn sniff(bytes: &[u8]) -> Container {
    if bytes.starts_with(b"PK\x03\x04") {
        return Container::Zip;
    }
    if bytes.starts_with(b"\x1f\x8b") {
        return Container::Gzip;
    }
    if is_tar(bytes) {
        return Container::Tar;
    }
    Container::Plain
}

/// A tar header is a 512-byte block whose checksum field (8 bytes at
/// offset 148) holds the unsigned sum of every header byte, computed with
/// the checksum field itself treated as spaces.
fn is_tar(bytes: &[u8]) -> bool {
    let Some(header) = bytes.get(..512) else {
        return false;
    };
    let checksum_field = &header[148..156];
    let Ok(text) = core::str::from_utf8(checksum_field) else {
        return false;
    };
    let trimmed = text.trim_matches(|c: char| c == '\0' || c == ' ');
    let Ok(recorded) = u32::from_str_radix(trimmed, 8) else {
        return false;
    };
    let sum: u32 = header
        .iter()
        .enumerate()
        .map(|(i, &b)| {
            if (148..156).contains(&i) {
                b' ' as u32
            } else {
                b as u32
            }
        })
        .sum();
    sum == recorded
}

/// Tracks limits shared across every file selected in one load operation.
pub(crate) struct LoadBudget {
    remaining_bytes: u64,
    remaining_entries: usize,
}

impl LoadBudget {
    pub(crate) fn new() -> Self {
        LoadBudget {
            remaining_bytes: DECOMPRESSION_BUDGET_BYTES,
            remaining_entries: MAX_ENTRIES,
        }
    }

    #[cfg(test)]
    fn with_limits(remaining_bytes: u64, remaining_entries: usize) -> Self {
        LoadBudget {
            remaining_bytes,
            remaining_entries,
        }
    }

    fn consume_bytes(&mut self, n: u64) -> Result<(), UnpackError> {
        self.remaining_bytes = self
            .remaining_bytes
            .checked_sub(n)
            .ok_or(UnpackError::BudgetExceeded)?;
        Ok(())
    }

    fn consume_entries(&mut self, n: usize) -> Result<(), UnpackError> {
        self.remaining_entries = self
            .remaining_entries
            .checked_sub(n)
            .ok_or(UnpackError::TooManyEntries)?;
        Ok(())
    }
}

/// Reads `reader` to the end in fixed-size chunks, charging each chunk to
/// `budget` before it is appended — so a decompression bomb is caught
/// mid-stream rather than after being materialized in full.
fn read_budgeted<R: Read>(
    mut reader: R,
    mut budget: Option<&mut LoadBudget>,
) -> Result<Vec<u8>, UnpackError> {
    let mut out = Vec::new();
    let mut chunk = [0u8; CHUNK_BYTES];
    loop {
        let n = reader.read(&mut chunk).map_err(UnpackError::Io)?;
        if n == 0 {
            break;
        }
        if let Some(budget) = &mut budget {
            budget.consume_bytes(n as u64)?;
        }
        out.extend_from_slice(&chunk[..n]);
    }
    Ok(out)
}

/// Strips any `..`, `.`, root or prefix components from an entry name, so
/// a tar path-traversal name (`../../etc/passwd`) becomes an inert,
/// displayable label instead of something that reads as an escaping path.
fn sanitize_name(raw: &str) -> String {
    let parts: Vec<&str> = std::path::Path::new(raw)
        .components()
        .filter_map(|c| match c {
            std::path::Component::Normal(s) => s.to_str(),
            _ => None,
        })
        .collect();
    if parts.is_empty() {
        "unnamed".to_string()
    } else {
        parts.join("/")
    }
}

/// Expands one file's bytes into items: split per line (skipping blanks
/// and `#` comments) when it's text, or a single base64/hex item when
/// it's a binary PSBT or transaction. Anything else becomes a single
/// base64 item, so downstream analysis reports it invalid rather than the
/// entry silently disappearing.
fn expand_entry(label: &str, data: &[u8]) -> Vec<UnpackedItem> {
    if let Ok(text) = core::str::from_utf8(data) {
        return tx_core::split_lines(text)
            .into_iter()
            .map(|line| UnpackedItem {
                label: label.to_string(),
                text: line.to_string(),
            })
            .collect();
    }
    let text = match tx_core::decode(data) {
        Ok(Decoded::Psbt(psbt)) => psbt.to_string(),
        Ok(Decoded::Transaction(tx)) => tx_core::serialize_hex(&tx),
        Err(_) => BASE64.encode(data),
    };
    vec![UnpackedItem {
        label: label.to_string(),
        text,
    }]
}

/// Sorts `(name, data)` pairs lexicographically by name, then expands each
/// into items in that order. This ordering is a documented guarantee —
/// the sequential broadcast loop depends on it.
fn expand_sorted(mut entries: Vec<(String, Vec<u8>)>) -> Vec<UnpackedItem> {
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    entries
        .into_iter()
        .flat_map(|(name, data)| expand_entry(&name, &data))
        .collect()
}

/// Whether `data` sniffs as a zip, gzip or tar container — used by the
/// caller to label a queued item's origin (`extracted from <name>` versus
/// `dropped file`) without re-deriving the detection logic.
pub fn is_archive(data: &[u8]) -> bool {
    sniff(data) != Container::Plain
}

fn unpack_zip(bytes: &[u8], budget: &mut LoadBudget) -> Result<Vec<UnpackedItem>, UnpackError> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(UnpackError::Zip)?;
    budget.consume_entries(archive.len())?;
    let mut entries = Vec::with_capacity(archive.len());
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(UnpackError::Zip)?;
        if file.is_dir() {
            continue;
        }
        let name = sanitize_name(file.name());
        let data = read_budgeted(&mut file, Some(budget))?;
        if is_archive(&data) {
            return Err(UnpackError::NestedArchive);
        }
        entries.push((name, data));
    }
    Ok(expand_sorted(entries))
}

fn unpack_tar_bytes(
    bytes: &[u8],
    budget: &mut LoadBudget,
    charge_bytes: bool,
) -> Result<Vec<UnpackedItem>, UnpackError> {
    let mut archive = tar::Archive::new(Cursor::new(bytes));
    let mut entries = Vec::new();
    for entry in archive.entries().map_err(UnpackError::Io)? {
        let mut entry = entry.map_err(UnpackError::Io)?;
        budget.consume_entries(1)?;
        if entry.header().entry_type().is_dir() {
            continue;
        }
        let path = entry.path().map_err(UnpackError::Io)?;
        let name = sanitize_name(&path.to_string_lossy());
        let data = read_budgeted(
            &mut entry,
            if charge_bytes {
                Some(&mut *budget)
            } else {
                None
            },
        )?;
        if is_archive(&data) {
            return Err(UnpackError::NestedArchive);
        }
        entries.push((name, data));
    }
    Ok(expand_sorted(entries))
}

fn unpack_gzip(
    name: &str,
    bytes: &[u8],
    budget: &mut LoadBudget,
) -> Result<Vec<UnpackedItem>, UnpackError> {
    let decoder = flate2::read::GzDecoder::new(bytes);
    let data = read_budgeted(decoder, Some(budget))?;
    match sniff(&data) {
        Container::Tar => unpack_tar_bytes(&data, budget, false),
        Container::Zip | Container::Gzip => Err(UnpackError::NestedArchive),
        Container::Plain => {
            budget.consume_entries(1)?;
            Ok(expand_entry(name, &data))
        }
    }
}

/// Unpacks one dropped or chosen file using limits shared by its load
/// operation. Sniffs `bytes` by magic bytes to decide whether it's an archive;
/// a non-archive is expanded on its own as a single item named `name`.
pub(crate) fn unpack(
    name: &str,
    bytes: &[u8],
    budget: &mut LoadBudget,
) -> Result<Vec<UnpackedItem>, UnpackError> {
    match sniff(bytes) {
        Container::Zip => unpack_zip(bytes, budget),
        Container::Gzip => unpack_gzip(name, bytes, budget),
        Container::Tar => unpack_tar_bytes(bytes, budget, true),
        Container::Plain => {
            budget.consume_bytes(bytes.len() as u64)?;
            Ok(expand_entry(name, bytes))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    const SAMPLE_TAR: &[u8] = include_bytes!("../tests/fixtures/sample.tar");
    const SAMPLE_TAR_GZ: &[u8] = include_bytes!("../tests/fixtures/sample.tar.gz");
    const SAMPLE_ZIP: &[u8] = include_bytes!("../tests/fixtures/sample.zip");
    const MULTILINE_TXT: &[u8] = include_bytes!("../tests/fixtures/multiline.txt");
    const NESTED_TAR: &[u8] = include_bytes!("../tests/fixtures/nested.tar");
    const MANY_ENTRIES_TAR: &[u8] = include_bytes!("../tests/fixtures/many_entries.tar");
    const HUGE_GZ: &[u8] = include_bytes!("../tests/fixtures/huge.gz");
    const TRAVERSAL_TAR: &[u8] = include_bytes!("../tests/fixtures/traversal.tar");
    const PSBT_SIGNED: &[u8] = include_bytes!("../tests/fixtures/psbt_signed.psbt");
    const PSBT_SIGNED_BASE64: &str = include_str!("../tests/fixtures/psbt_signed.base64");

    fn unpack_file(name: &str, bytes: &[u8]) -> Result<Vec<UnpackedItem>, UnpackError> {
        unpack(name, bytes, &mut LoadBudget::new())
    }

    /// `sample.{tar,tar.gz,zip}` all hold the same three entries —
    /// `charlie.txt`, `alpha.txt`, `bravo.txt`, in that (non-lexicographic)
    /// insertion order — so the expected result is identical: three items
    /// whose text reflects alphabetical-by-name order.
    fn assert_three_in_lex_order(items: &[UnpackedItem]) {
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].text, "alpha-content");
        assert_eq!(items[1].text, "bravo-content");
        assert_eq!(items[2].text, "charlie-content");
        assert_eq!(items[0].label, "alpha.txt");
        assert_eq!(items[1].label, "bravo.txt");
        assert_eq!(items[2].label, "charlie.txt");
    }

    #[wasm_bindgen_test]
    fn tar_entries_sorted_lexicographically() {
        let items = unpack_file("sample.tar", SAMPLE_TAR).unwrap();
        assert_three_in_lex_order(&items);
    }

    #[wasm_bindgen_test]
    fn tar_gz_entries_sorted_lexicographically() {
        let items = unpack_file("sample.tar.gz", SAMPLE_TAR_GZ).unwrap();
        assert_three_in_lex_order(&items);
    }

    #[wasm_bindgen_test]
    fn zip_entries_sorted_lexicographically() {
        let items = unpack_file("sample.zip", SAMPLE_ZIP).unwrap();
        assert_three_in_lex_order(&items);
    }

    #[wasm_bindgen_test]
    fn text_file_splits_per_line_skipping_blanks_and_comments() {
        let items = unpack_file("multiline.txt", MULTILINE_TXT).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].text, "first-line");
        assert_eq!(items[1].text, "second-line");
        assert!(items.iter().all(|item| item.label == "multiline.txt"));
    }

    #[wasm_bindgen_test]
    fn binary_psbt_entry_becomes_base64_item() {
        let items = unpack_file("psbt_signed.psbt", PSBT_SIGNED).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].text, PSBT_SIGNED_BASE64.trim());
    }

    #[wasm_bindgen_test]
    fn archive_with_too_many_entries_is_refused() {
        let err = unpack_file("many_entries.tar", MANY_ENTRIES_TAR).unwrap_err();
        assert!(matches!(err, UnpackError::TooManyEntries));
    }

    #[wasm_bindgen_test]
    fn entry_exceeding_the_decompression_budget_is_refused() {
        let err = unpack_file("huge.gz", HUGE_GZ).unwrap_err();
        assert!(matches!(err, UnpackError::BudgetExceeded));
    }

    #[wasm_bindgen_test]
    fn tar_gz_content_is_charged_once() {
        let mut decoder = flate2::read::GzDecoder::new(SAMPLE_TAR_GZ);
        let mut tar_bytes = Vec::new();
        decoder.read_to_end(&mut tar_bytes).unwrap();
        let mut budget = LoadBudget::with_limits(tar_bytes.len() as u64, MAX_ENTRIES);

        let items = unpack_gzip("sample.tar.gz", SAMPLE_TAR_GZ, &mut budget).unwrap();

        assert_three_in_lex_order(&items);
        assert_eq!(budget.remaining_bytes, 0);
    }

    #[wasm_bindgen_test]
    fn two_archives_share_decompressed_byte_budget() {
        let mut archive = zip::ZipArchive::new(Cursor::new(SAMPLE_ZIP)).unwrap();
        let unpacked_bytes = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().size())
            .sum::<u64>();
        let mut budget = LoadBudget::with_limits(unpacked_bytes * 2 - 1, MAX_ENTRIES);

        unpack("first.zip", SAMPLE_ZIP, &mut budget).unwrap();
        let err = unpack("second.zip", SAMPLE_ZIP, &mut budget).unwrap_err();

        assert!(matches!(err, UnpackError::BudgetExceeded));
    }

    #[wasm_bindgen_test]
    fn two_archives_share_entry_limit() {
        let mut budget = LoadBudget::with_limits(DECOMPRESSION_BUDGET_BYTES, 5);

        unpack("first.zip", SAMPLE_ZIP, &mut budget).unwrap();
        let err = unpack("second.zip", SAMPLE_ZIP, &mut budget).unwrap_err();

        assert!(matches!(err, UnpackError::TooManyEntries));
    }

    #[wasm_bindgen_test]
    fn nested_archive_is_refused() {
        let err = unpack_file("nested.tar", NESTED_TAR).unwrap_err();
        assert!(matches!(err, UnpackError::NestedArchive));
    }

    #[wasm_bindgen_test]
    fn tar_entry_with_traversal_name_is_sanitized() {
        let items = unpack_file("traversal.tar", TRAVERSAL_TAR).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "etc/passwd");
        assert!(!items[0].label.contains(".."));
    }

    #[wasm_bindgen_test]
    fn plain_file_is_a_single_entry() {
        let items = unpack_file("tx.hex", b"not a recognised anything").unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "tx.hex");
        assert_eq!(items[0].text, "not a recognised anything");
    }

    #[wasm_bindgen_test]
    fn sanitize_name_strips_traversal_components() {
        assert_eq!(sanitize_name("../../etc/passwd"), "etc/passwd");
        assert_eq!(sanitize_name("plain.txt"), "plain.txt");
        assert_eq!(sanitize_name("/abs/path"), "abs/path");
        assert_eq!(sanitize_name("./a/../b"), "a/b");
    }

    #[wasm_bindgen_test]
    fn is_tar_rejects_non_tar_bytes() {
        assert!(!is_tar(b"PK\x03\x04 not actually a tar"));
        assert!(!is_tar(&[0u8; 511]));
    }
}
