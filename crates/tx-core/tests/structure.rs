use bitcoin::Transaction;
use tx_core::{Decoded, StructuralError, check_structure, decode};

const TX_HEX: &str = include_str!("fixtures/tx.hex");

fn fixture_tx() -> Transaction {
    match decode(TX_HEX.as_bytes()).expect("fixture decodes") {
        Decoded::Transaction(tx) => tx,
        Decoded::Psbt(_) => panic!("fixture is a raw transaction"),
    }
}

#[test]
fn accepts_a_transaction_with_inputs_and_outputs() {
    assert_eq!(check_structure(&fixture_tx()), Ok(()));
}

#[test]
fn rejects_a_transaction_with_no_inputs() {
    let mut tx = fixture_tx();
    tx.input.clear();
    assert_eq!(check_structure(&tx), Err(StructuralError::NoInputs));
}

#[test]
fn rejects_a_transaction_with_no_outputs() {
    let mut tx = fixture_tx();
    tx.output.clear();
    assert_eq!(check_structure(&tx), Err(StructuralError::NoOutputs));
}

#[test]
fn reports_the_missing_inputs_first_when_both_are_empty() {
    let mut tx = fixture_tx();
    tx.input.clear();
    tx.output.clear();
    assert_eq!(check_structure(&tx), Err(StructuralError::NoInputs));
}

#[test]
fn error_messages_name_what_is_missing() {
    assert_eq!(
        StructuralError::NoInputs.to_string(),
        "transaction has no inputs"
    );
    assert_eq!(
        StructuralError::NoOutputs.to_string(),
        "transaction has no outputs"
    );
}
