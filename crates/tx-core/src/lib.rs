//! Format detection and decoding for PSBTs and raw Bitcoin transactions.
//!
//! Everything here works on plain bytes: no filesystem, no network, no
//! filename or extension ever enters the decision. This is what turns
//! arbitrary user input (a paste, a dropped file, an archive entry) into a
//! typed `bitcoin` value, or a precise, displayable error.

use core::fmt;
use core::str::{self, FromStr};

use bitcoin::hex::FromHex;
use bitcoin::psbt::PsbtParseError;
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Psbt, Transaction, consensus};
use miniscript::psbt::PsbtExt;

/// Leading magic of a raw binary PSBT: ASCII `psbt` followed by `0xff`.
const PSBT_MAGIC: &[u8] = b"psbt\xff";

/// The four accepted encodings for a transaction or PSBT, detected by
/// content only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// Raw binary PSBT, leading magic `psbt\xff`.
    PsbtBinary,
    /// Base64-encoded PSBT text.
    PsbtBase64,
    /// Hex-encoded transaction text.
    TxHex,
    /// Raw binary, consensus-encoded transaction.
    TxBinary,
}

/// A successfully decoded PSBT or transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decoded {
    Psbt(Psbt),
    Transaction(Transaction),
}

/// Everything that can go wrong turning input bytes into a [`Decoded`]
/// value. Every variant renders a message fit for a queue row's error note
/// card.
#[derive(Debug)]
pub enum TxCoreError {
    /// The input matches none of the four accepted formats.
    UnrecognisedFormat,
    /// Text claiming to be a base64-encoded PSBT is not valid base64.
    MalformedBase64(bitcoin::base64::DecodeError),
    /// Text claiming to be hex-encoded is not valid hex.
    MalformedHex(bitcoin::hex::HexToBytesError),
    /// The bytes are not a well-formed PSBT.
    PsbtDeserialize(bitcoin::psbt::Error),
    /// The bytes are not a well-formed transaction.
    TransactionDeserialize(consensus::encode::Error),
}

impl fmt::Display for TxCoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TxCoreError::UnrecognisedFormat => {
                write!(f, "not a recognised PSBT or transaction")
            }
            TxCoreError::MalformedBase64(e) => write!(f, "not valid base64: {e}"),
            TxCoreError::MalformedHex(e) => write!(f, "not valid hex: {e}"),
            TxCoreError::PsbtDeserialize(e) => write!(f, "malformed PSBT: {e}"),
            TxCoreError::TransactionDeserialize(e) => write!(f, "malformed transaction: {e}"),
        }
    }
}

impl std::error::Error for TxCoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TxCoreError::UnrecognisedFormat => None,
            TxCoreError::MalformedBase64(e) => Some(e),
            TxCoreError::MalformedHex(e) => Some(e),
            TxCoreError::PsbtDeserialize(e) => Some(e),
            TxCoreError::TransactionDeserialize(e) => Some(e),
        }
    }
}

/// Detects the format of `input` by content only: never by filename or
/// extension. Checks, in order, the binary PSBT magic, a hex-decoded PSBT
/// magic (ruled out rather than misread as a transaction), a base64 PSBT,
/// then consensus deserialization for the hex and binary transaction cases.
pub fn detect(input: &[u8]) -> Option<Format> {
    let trimmed = input.trim_ascii();

    if trimmed.starts_with(PSBT_MAGIC) {
        return Some(Format::PsbtBinary);
    }

    if let Ok(text) = str::from_utf8(trimmed) {
        let text = text.trim();
        let hex_bytes = Vec::<u8>::from_hex(text).ok();

        if hex_bytes
            .as_deref()
            .is_some_and(|bytes| bytes.starts_with(PSBT_MAGIC))
        {
            return None;
        }
        if Psbt::from_str(text).is_ok() {
            return Some(Format::PsbtBase64);
        }
        if hex_bytes.is_some_and(|bytes| consensus::deserialize::<Transaction>(&bytes).is_ok()) {
            return Some(Format::TxHex);
        }
    }

    consensus::deserialize::<Transaction>(trimmed)
        .ok()
        .map(|_| Format::TxBinary)
}

/// Detects the format of `input`, then decodes it into a [`Decoded`] value.
pub fn decode(input: &[u8]) -> Result<Decoded, TxCoreError> {
    let format = detect(input).ok_or(TxCoreError::UnrecognisedFormat)?;
    decode_as(format, input)
}

/// Decodes `input` as the given `format`, without re-detecting it.
pub fn decode_as(format: Format, input: &[u8]) -> Result<Decoded, TxCoreError> {
    let trimmed = input.trim_ascii();
    match format {
        Format::PsbtBinary => decode_psbt_binary(trimmed),
        Format::PsbtBase64 => decode_psbt_base64(trimmed),
        Format::TxHex => decode_tx_hex(trimmed),
        Format::TxBinary => decode_tx_binary(trimmed),
    }
}

fn decode_psbt_binary(bytes: &[u8]) -> Result<Decoded, TxCoreError> {
    Psbt::deserialize(bytes)
        .map(Decoded::Psbt)
        .map_err(TxCoreError::PsbtDeserialize)
}

fn decode_psbt_base64(bytes: &[u8]) -> Result<Decoded, TxCoreError> {
    let text = str::from_utf8(bytes)
        .map_err(|_| TxCoreError::UnrecognisedFormat)?
        .trim();
    Psbt::from_str(text)
        .map(Decoded::Psbt)
        .map_err(|e| match e {
            PsbtParseError::Base64Encoding(err) => TxCoreError::MalformedBase64(err),
            PsbtParseError::PsbtEncoding(err) => TxCoreError::PsbtDeserialize(err),
            _ => TxCoreError::UnrecognisedFormat,
        })
}

fn decode_tx_hex(bytes: &[u8]) -> Result<Decoded, TxCoreError> {
    let text = str::from_utf8(bytes)
        .map_err(|_| TxCoreError::UnrecognisedFormat)?
        .trim();
    let decoded = Vec::<u8>::from_hex(text).map_err(TxCoreError::MalformedHex)?;
    consensus::deserialize(&decoded)
        .map(Decoded::Transaction)
        .map_err(TxCoreError::TransactionDeserialize)
}

fn decode_tx_binary(bytes: &[u8]) -> Result<Decoded, TxCoreError> {
    consensus::deserialize(bytes)
        .map(Decoded::Transaction)
        .map_err(TxCoreError::TransactionDeserialize)
}

/// Splits multi-entry text input into individual entries: one per non-blank
/// line, skipping lines whose first non-whitespace character is `#`.
pub fn split_lines(input: &str) -> Vec<&str> {
    input
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect()
}

/// The signature state of one PSBT input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputSignatureState {
    /// `final_script_sig` or `final_script_witness` is already present.
    pub finalized: bool,
    /// Number of partial signatures collected so far.
    pub partial_sig_count: usize,
}

/// Per-input signature analysis of a PSBT, in input order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureAnalysis {
    pub inputs: Vec<InputSignatureState>,
}

impl SignatureAnalysis {
    /// Number of inputs not yet finalized.
    pub fn incomplete_inputs(&self) -> usize {
        self.inputs.iter().filter(|input| !input.finalized).count()
    }
}

/// Reports, for each input of `psbt`, whether it is already finalized or
/// how many partial signatures it carries.
pub fn analyze_signatures(psbt: &Psbt) -> SignatureAnalysis {
    let inputs = psbt
        .inputs
        .iter()
        .map(|input| InputSignatureState {
            finalized: input.final_script_sig.is_some() || input.final_script_witness.is_some(),
            partial_sig_count: input.partial_sigs.len(),
        })
        .collect();
    SignatureAnalysis { inputs }
}

/// A PSBT could not be finalized because one or more inputs are missing
/// signatures. Carries how many; the item must not reach the queue.
#[derive(Debug)]
pub struct IncompleteSignaturesError {
    pub incomplete_inputs: usize,
}

impl fmt::Display for IncompleteSignaturesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PSBT cannot be finalized: {} input(s) missing signatures",
            self.incomplete_inputs
        )
    }
}

impl std::error::Error for IncompleteSignaturesError {}

/// Finalizes `psbt` and extracts its network-serialized transaction.
///
/// Already-finalized inputs are left untouched, inputs with complete
/// signatures are finalized in place, and if any input is still missing
/// signatures afterward this returns [`IncompleteSignaturesError`] carrying
/// how many rather than extracting a partial transaction.
///
/// Uses [`Psbt::extract_tx_unchecked_fee_rate`], not `extract_tx`: the
/// latter refuses transactions whose fee rate it judges absurd, which is a
/// fee gate this project does not apply anywhere else.
pub fn finalize(mut psbt: Psbt) -> Result<Transaction, IncompleteSignaturesError> {
    let secp = Secp256k1::new();
    if let Err(errors) = psbt.finalize_mut(&secp) {
        return Err(IncompleteSignaturesError {
            incomplete_inputs: errors.len(),
        });
    }
    Ok(psbt.extract_tx_unchecked_fee_rate())
}

/// Network-serializes `tx`: the bytes ever sent to the server.
pub fn serialize(tx: &Transaction) -> Vec<u8> {
    consensus::serialize(tx)
}

/// Hex-encodes the network serialization of `tx`.
pub fn serialize_hex(tx: &Transaction) -> String {
    consensus::encode::serialize_hex(tx)
}
