use std::collections::BTreeMap;

use bitcoin::absolute::LockTime;
use bitcoin::bip32::{DerivationPath, Fingerprint};
use bitcoin::hashes::Hash;
use bitcoin::psbt::Input;
use bitcoin::secp256k1::{Secp256k1, SecretKey};
use bitcoin::transaction::Version;
use bitcoin::{
    Amount, Network, OutPoint, PrivateKey, Psbt, PublicKey, ScriptBuf, Sequence, Transaction, TxIn,
    TxOut, Txid, Witness,
};
use tx_core::{IncompleteSignaturesError, analyze_signatures, finalize, serialize, serialize_hex};

const TX_HEX: &str = include_str!("fixtures/tx.hex");
const TX_BINARY: &[u8] = include_bytes!("fixtures/tx.bin");

/// Value of each test input, unless a test overrides it.
const NORMAL_INPUT_VALUE: Amount = Amount::from_sat(100_000);
/// Output value that leaves an ordinary fee against `NORMAL_INPUT_VALUE`.
const NORMAL_OUTPUT_VALUE: Amount = Amount::from_sat(90_000);

/// One P2WPKH key: a distinct, deterministic private key and its address.
struct TestKey {
    private_key: PrivateKey,
    public_key: PublicKey,
    script_pubkey: ScriptBuf,
}

fn test_key(secp: &Secp256k1<bitcoin::secp256k1::All>, index: u8) -> TestKey {
    let secret_key = SecretKey::from_slice(&[index + 1; 32]).expect("valid secret key");
    let private_key = PrivateKey::new(secret_key, Network::Bitcoin);
    let public_key = private_key.public_key(secp);
    let script_pubkey = ScriptBuf::new_p2wpkh(&public_key.wpubkey_hash().expect("compressed key"));
    TestKey {
        private_key,
        public_key,
        script_pubkey,
    }
}

/// Builds an unsigned PSBT with one P2WPKH input per entry of `sign`, each
/// worth `input_value`, and a single output worth `output_value`. Signs (via
/// `Psbt::sign`) exactly the inputs whose entry is `true`. Every input has a
/// distinct deterministic key, so signing one never satisfies another.
fn p2wpkh_psbt(sign: &[bool], input_value: Amount, output_value: Amount) -> Psbt {
    let secp = Secp256k1::new();
    let mut keymap: BTreeMap<PublicKey, PrivateKey> = BTreeMap::new();
    let mut tx_ins = Vec::new();
    let mut inputs = Vec::new();

    for (index, &should_sign) in sign.iter().enumerate() {
        let key = test_key(&secp, index as u8);

        tx_ins.push(TxIn {
            previous_output: OutPoint::new(Txid::all_zeros(), index as u32),
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: Witness::new(),
        });

        let mut input = Input {
            witness_utxo: Some(TxOut {
                value: input_value,
                script_pubkey: key.script_pubkey,
            }),
            ..Default::default()
        };
        if should_sign {
            keymap.insert(key.public_key, key.private_key);
            input.bip32_derivation.insert(
                key.public_key.inner,
                (Fingerprint::default(), DerivationPath::default()),
            );
        }
        inputs.push(input);
    }

    let output_key = test_key(&secp, 99);
    let unsigned_tx = Transaction {
        version: Version::TWO,
        lock_time: LockTime::ZERO,
        input: tx_ins,
        output: vec![TxOut {
            value: output_value,
            script_pubkey: output_key.script_pubkey,
        }],
    };

    let mut psbt = Psbt::from_unsigned_tx(unsigned_tx).expect("valid unsigned tx");
    psbt.inputs = inputs;
    // Some entries are deliberately left unsigned; ignore the partial error.
    let _ = psbt.sign(&keymap, &secp);
    psbt
}

#[test]
fn complete_signatures_finalizes_and_extracts() {
    let psbt = p2wpkh_psbt(&[true], NORMAL_INPUT_VALUE, NORMAL_OUTPUT_VALUE);
    let analysis = analyze_signatures(&psbt);
    assert_eq!(analysis.inputs.len(), 1);
    assert!(!analysis.inputs[0].finalized);
    assert_eq!(analysis.inputs[0].partial_sig_count, 1);
    assert_eq!(
        analysis.incomplete_inputs(),
        1,
        "signed but not yet finalized"
    );

    let tx = finalize(psbt).expect("complete PSBT finalizes");
    assert_eq!(tx.input.len(), 1);
    assert_eq!(tx.input[0].witness.len(), 2, "signature and pubkey");
    assert_eq!(
        tx.compute_txid().to_string(),
        "4c3b230fe3b4dceeee47a3642e27c75b7b7011fcb46d0bec1cffb18d1559fa8f"
    );
}

#[test]
fn already_finalized_extracts_directly_with_known_txid() {
    let psbt = p2wpkh_psbt(&[true], NORMAL_INPUT_VALUE, NORMAL_OUTPUT_VALUE);
    let finalized_tx = finalize(psbt.clone()).expect("complete PSBT finalizes");

    // Build a second PSBT that starts already finalized: the final witness
    // is present from the start, no partial signature ever recorded.
    let mut already_finalized = psbt;
    already_finalized.inputs[0].partial_sigs.clear();
    already_finalized.inputs[0].bip32_derivation.clear();
    already_finalized.inputs[0].final_script_witness = Some(finalized_tx.input[0].witness.clone());

    let analysis = analyze_signatures(&already_finalized);
    assert!(analysis.inputs[0].finalized);
    assert_eq!(analysis.incomplete_inputs(), 0);

    let tx = finalize(already_finalized).expect("already-finalized PSBT extracts");
    assert_eq!(
        tx.compute_txid().to_string(),
        "4c3b230fe3b4dceeee47a3642e27c75b7b7011fcb46d0bec1cffb18d1559fa8f"
    );
    assert_eq!(tx.compute_txid(), finalized_tx.compute_txid());
}

#[test]
fn missing_signature_on_one_of_two_inputs_reports_incomplete_count() {
    let psbt = p2wpkh_psbt(&[true, false], NORMAL_INPUT_VALUE, NORMAL_OUTPUT_VALUE);
    let analysis = analyze_signatures(&psbt);
    assert_eq!(
        analysis.incomplete_inputs(),
        2,
        "neither input finalized yet"
    );

    let err = finalize(psbt).expect_err("one input has no signature");
    assert_eq!(err.incomplete_inputs, 1);
}

#[test]
fn incomplete_signatures_error_displays_the_count() {
    let err = IncompleteSignaturesError {
        incomplete_inputs: 3,
    };
    assert_eq!(
        err.to_string(),
        "PSBT cannot be finalized: 3 input(s) missing signatures"
    );
}

#[test]
fn raw_transaction_txid_and_serialization_round_trip_unchanged() {
    let tx: Transaction = bitcoin::consensus::deserialize(TX_BINARY).expect("valid fixture tx");

    assert_eq!(serialize(&tx), TX_BINARY);
    assert_eq!(serialize_hex(&tx), TX_HEX.trim());
    assert_eq!(
        tx.compute_txid().to_string(),
        "a6eab3c14ab5272a58a5ba91505ba1a4b6d7a3a9fcbd187b6cd99a7b6d548cb7"
    );
}

#[test]
fn extract_then_reparse_round_trips_to_the_same_txid() {
    let psbt = p2wpkh_psbt(&[true], NORMAL_INPUT_VALUE, NORMAL_OUTPUT_VALUE);
    let tx = finalize(psbt).expect("complete PSBT finalizes");
    let txid = tx.compute_txid();

    let raw = serialize(&tx);
    let reparsed: Transaction = bitcoin::consensus::deserialize(&raw).expect("round-trips");

    assert_eq!(reparsed.compute_txid(), txid);
    assert_eq!(serialize(&reparsed), raw);
}

#[test]
fn high_fee_rate_psbt_extracts_successfully() {
    // A ~110 vbyte transaction paying a fee of ~10_000_000 sats is well over
    // the 25_000 sat/vB threshold `Psbt::extract_tx` refuses as absurd.
    // `finalize` must still extract it: nothing in this project gates on fee.
    let psbt = p2wpkh_psbt(&[true], Amount::from_sat(10_000_100), Amount::from_sat(100));
    let tx = finalize(psbt).expect("high-fee PSBT still finalizes and extracts");
    assert_eq!(tx.output[0].value, Amount::from_sat(100));
}

#[test]
fn finalization_is_deterministic() {
    let first = finalize(p2wpkh_psbt(
        &[true],
        NORMAL_INPUT_VALUE,
        NORMAL_OUTPUT_VALUE,
    ))
    .expect("finalizes");
    let second = finalize(p2wpkh_psbt(
        &[true],
        NORMAL_INPUT_VALUE,
        NORMAL_OUTPUT_VALUE,
    ))
    .expect("finalizes");
    assert_eq!(first, second);
}
