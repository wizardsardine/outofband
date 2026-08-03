use bitcoin::Transaction;
use tx_core::{Decoded, Format, TxCoreError, decode, decode_as, detect, split_lines};

const PSBT_BASE64: &str = include_str!("fixtures/psbt_signed.base64");
const PSBT_BINARY: &[u8] = include_bytes!("fixtures/psbt_signed.psbt");
const PSBT_TRUNCATED: &[u8] = include_bytes!("fixtures/psbt_truncated.psbt");
const TX_HEX: &str = include_str!("fixtures/tx.hex");
const TX_BINARY: &[u8] = include_bytes!("fixtures/tx.bin");
const MULTI_BLOB: &str = include_str!("fixtures/multi.txt");

struct Case {
    name: &'static str,
    input: &'static [u8],
    expected_format: Option<Format>,
}

#[test]
fn detects_each_accepted_format() {
    let cases = [
        Case {
            name: "psbt binary",
            input: PSBT_BINARY,
            expected_format: Some(Format::PsbtBinary),
        },
        Case {
            name: "psbt base64",
            input: PSBT_BASE64.as_bytes(),
            expected_format: Some(Format::PsbtBase64),
        },
        Case {
            name: "tx hex",
            input: TX_HEX.as_bytes(),
            expected_format: Some(Format::TxHex),
        },
        Case {
            name: "tx binary",
            input: TX_BINARY,
            expected_format: Some(Format::TxBinary),
        },
        Case {
            name: "empty input",
            input: b"",
            expected_format: None,
        },
        Case {
            name: "truncated psbt still carries the magic",
            input: PSBT_TRUNCATED,
            expected_format: Some(Format::PsbtBinary),
        },
        Case {
            name: "odd length hex",
            input: b"abc",
            expected_format: None,
        },
    ];

    for case in cases {
        let format = detect(case.input);
        assert_eq!(format, case.expected_format, "case: {}", case.name);
    }
}

#[test]
fn decodes_psbt_binary() {
    match decode(PSBT_BINARY).expect("psbt binary decodes") {
        Decoded::Psbt(_) => {}
        Decoded::Transaction(_) => panic!("expected a PSBT"),
    }
}

#[test]
fn decodes_psbt_base64() {
    match decode(PSBT_BASE64.as_bytes()).expect("psbt base64 decodes") {
        Decoded::Psbt(_) => {}
        Decoded::Transaction(_) => panic!("expected a PSBT"),
    }
}

#[test]
fn decodes_tx_hex() {
    match decode(TX_HEX.as_bytes()).expect("tx hex decodes") {
        Decoded::Transaction(tx) => {
            let expected: Transaction = bitcoin::consensus::deserialize(TX_BINARY).unwrap();
            assert_eq!(tx, expected);
        }
        Decoded::Psbt(_) => panic!("expected a transaction"),
    }
}

#[test]
fn decodes_tx_binary() {
    match decode(TX_BINARY).expect("tx binary decodes") {
        Decoded::Transaction(_) => {}
        Decoded::Psbt(_) => panic!("expected a transaction"),
    }
}

#[test]
fn empty_input_is_unrecognised() {
    let err = decode(b"").unwrap_err();
    assert!(matches!(err, TxCoreError::UnrecognisedFormat));
}

#[test]
fn truncated_psbt_fails_deserialization() {
    let err = decode(PSBT_TRUNCATED).unwrap_err();
    assert!(matches!(err, TxCoreError::PsbtDeserialize(_)), "got: {err}");
}

#[test]
fn odd_length_hex_forced_as_tx_hex_is_malformed_hex() {
    let err = decode_as(Format::TxHex, b"abc").unwrap_err();
    assert!(matches!(err, TxCoreError::MalformedHex(_)), "got: {err}");
}

#[test]
fn invalid_base64_forced_as_psbt_base64_is_malformed_base64() {
    let err = decode_as(Format::PsbtBase64, b"not!valid@base64").unwrap_err();
    assert!(matches!(err, TxCoreError::MalformedBase64(_)), "got: {err}");
}

#[test]
fn base64_decoding_to_garbage_is_a_psbt_deserialize_error() {
    // Valid base64, but the decoded bytes are not a PSBT: no magic, no
    // structure. Exercises the "decodes to garbage" fixture case.
    let err = decode_as(Format::PsbtBase64, b"aGVsbG8gd29ybGQ=").unwrap_err();
    assert!(matches!(err, TxCoreError::PsbtDeserialize(_)), "got: {err}");
}

#[test]
fn malformed_tx_hex_is_a_transaction_deserialize_error() {
    // Valid, even-length hex, but not a well-formed transaction.
    let err = decode_as(Format::TxHex, b"deadbeef").unwrap_err();
    assert!(
        matches!(err, TxCoreError::TransactionDeserialize(_)),
        "got: {err}"
    );
}

#[test]
fn malformed_tx_binary_is_a_transaction_deserialize_error() {
    let err = decode_as(Format::TxBinary, &[0x00, 0x01, 0x02]).unwrap_err();
    assert!(
        matches!(err, TxCoreError::TransactionDeserialize(_)),
        "got: {err}"
    );
}

#[test]
fn hex_text_of_a_psbt_is_not_recognised_as_a_transaction() {
    // The ASCII hex of raw PSBT bytes hex-decodes to something starting
    // with the PSBT magic; it must never be misread as TxHex.
    let hex_of_psbt = bitcoin::consensus::encode::serialize_hex(&PsbtHexProbe(PSBT_BINARY));
    assert_eq!(detect(hex_of_psbt.as_bytes()), None);
}

// Wraps raw bytes so `serialize_hex` can hex-encode them directly without
// pulling in a second hex-encoding dependency just for this one test.
struct PsbtHexProbe<'a>(&'a [u8]);

impl bitcoin::consensus::Encodable for PsbtHexProbe<'_> {
    fn consensus_encode<W: bitcoin::io::Write + ?Sized>(
        &self,
        writer: &mut W,
    ) -> Result<usize, bitcoin::io::Error> {
        writer.write_all(self.0)?;
        Ok(self.0.len())
    }
}

#[test]
fn split_lines_skips_blanks_and_comments() {
    let lines = split_lines(MULTI_BLOB);

    assert_eq!(lines.len(), 3);
    assert!(lines[0].starts_with("0100000001"));
    assert!(lines[1].starts_with("cHNidP8"));
    assert_eq!(lines[2], "not a valid line at all");
}

#[test]
fn split_multi_line_blob_mixes_formats_and_comments() {
    let lines = split_lines(MULTI_BLOB);

    assert_eq!(detect(lines[0].as_bytes()), Some(Format::TxHex));
    assert_eq!(detect(lines[1].as_bytes()), Some(Format::PsbtBase64));
    assert_eq!(detect(lines[2].as_bytes()), None);
}
