use bitcoin::absolute::LockTime;
use bitcoin::hashes::Hash;
use bitcoin::psbt::Input;
use bitcoin::transaction::Version;
use bitcoin::{Amount, OutPoint, Psbt, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid};
use tx_core::{
    AmountOverflowError, FinalizedPsbt, FloorComparison, InputSum, PsbtAnalysisError, PsbtFee,
    PsbtValidationError, compare_to_floor, fee_from_user_total, fee_rate, finalize, output_sum,
    psbt_fee, psbt_input_sum, psbt_output_sum, vsize,
};

/// A previous output an input can spend: either UTXO data or none at all.
enum Utxo {
    Witness(u64),
    NonWitness(u64),
    Missing,
}

/// Builds an unsigned PSBT with one dummy P2WSH input per entry of `inputs`
/// and one output per entry of `outputs`. Each input's UTXO data (or lack
/// of it) is set per `Utxo`; nothing is signed or finalized, since fee and
/// output-sum extraction never depend on signature state.
fn psbt(inputs: &[Utxo], outputs: &[u64]) -> Psbt {
    let script_pubkey =
        ScriptBuf::from_hex("0020000000000000000000000000000000000000000000000000000000000000")
            .expect("valid script");

    // Every dummy previous transaction below has exactly one output, so
    // every input spends vout 0 of it regardless of the input's own index.
    let tx_ins: Vec<TxIn> = (0..inputs.len())
        .map(|_| TxIn {
            previous_output: OutPoint::new(Txid::all_zeros(), 0),
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: bitcoin::Witness::new(),
        })
        .collect();

    let unsigned_tx = Transaction {
        version: Version::TWO,
        lock_time: LockTime::ZERO,
        input: tx_ins,
        output: outputs
            .iter()
            .map(|&value| TxOut {
                value: Amount::from_sat(value),
                script_pubkey: script_pubkey.clone(),
            })
            .collect(),
    };

    let mut psbt = Psbt::from_unsigned_tx(unsigned_tx).expect("valid unsigned tx");
    psbt.inputs = inputs
        .iter()
        .map(|utxo| match utxo {
            Utxo::Witness(value) => Input {
                witness_utxo: Some(TxOut {
                    value: Amount::from_sat(*value),
                    script_pubkey: script_pubkey.clone(),
                }),
                ..Default::default()
            },
            Utxo::NonWitness(value) => {
                let prev_tx = Transaction {
                    version: Version::TWO,
                    lock_time: LockTime::ZERO,
                    input: vec![],
                    output: vec![TxOut {
                        value: Amount::from_sat(*value),
                        script_pubkey: script_pubkey.clone(),
                    }],
                };
                Input {
                    non_witness_utxo: Some(prev_tx),
                    ..Default::default()
                }
            }
            Utxo::Missing => Input::default(),
        })
        .collect();
    for (tx_in, input) in psbt.unsigned_tx.input.iter_mut().zip(psbt.inputs.iter()) {
        if let Some(previous_tx) = &input.non_witness_utxo {
            tx_in.previous_output.txid = previous_tx.compute_txid();
        }
    }
    psbt
}

#[test]
fn psbt_with_witness_utxo_data_yields_exact_fee_and_output_sum() {
    let psbt = psbt(&[Utxo::Witness(100_000), Utxo::Witness(50_000)], &[120_000]);

    assert_eq!(psbt_input_sum(&psbt), Ok(InputSum::Known(150_000)));
    assert_eq!(psbt_output_sum(&psbt), Ok(120_000));
    assert_eq!(psbt_fee(&psbt), Ok(PsbtFee::Known { fee_sats: 30_000 }));
}

#[test]
fn psbt_with_non_witness_utxo_data_yields_exact_fee_and_output_sum() {
    let psbt = psbt(&[Utxo::NonWitness(200_000)], &[150_000]);

    assert_eq!(psbt_input_sum(&psbt), Ok(InputSum::Known(200_000)));
    assert_eq!(psbt_output_sum(&psbt), Ok(150_000));
    assert_eq!(psbt_fee(&psbt), Ok(PsbtFee::Known { fee_sats: 50_000 }));
}

#[test]
fn psbt_missing_utxo_data_on_one_of_three_inputs_reports_unknown_with_count() {
    let psbt = psbt(
        &[
            Utxo::Witness(100_000),
            Utxo::Missing,
            Utxo::NonWitness(50_000),
        ],
        &[80_000],
    );

    assert_eq!(
        psbt_input_sum(&psbt),
        Ok(InputSum::Unknown {
            missing_utxo_inputs: 1
        })
    );
    assert_eq!(
        psbt_fee(&psbt),
        Ok(PsbtFee::Unknown {
            missing_utxo_inputs: 1
        })
    );
    // Output sum is derivable from the unsigned tx regardless of the
    // inputs' UTXO state: a partial-UTXO PSBT is never stranded without it.
    assert_eq!(psbt_output_sum(&psbt), Ok(80_000));
}

#[test]
fn raw_transaction_has_known_output_sum_but_no_automatic_fee() {
    let tx = Transaction {
        version: Version::TWO,
        lock_time: LockTime::ZERO,
        input: vec![],
        output: vec![
            TxOut {
                value: Amount::from_sat(12_345),
                script_pubkey: ScriptBuf::new(),
            },
            TxOut {
                value: Amount::from_sat(6_789),
                script_pubkey: ScriptBuf::new(),
            },
        ],
    };

    assert_eq!(output_sum(&tx), Ok(19_134));
    // Previous outputs are never known for a raw transaction, so there is
    // no automatic fee — only fee_from_user_total (the inline "total input
    // value" path) can produce one.
    assert_eq!(compare_to_floor(None, 4.0), FloorComparison::Unknown);
}

#[test]
fn user_supplied_total_derives_fee_and_rate_matching_hand_computation() {
    let output_sum = 100_000u64;
    let total_input = 105_000u64;
    let vsize = 140u64;

    let fee = fee_from_user_total(total_input, output_sum);
    assert_eq!(fee, 5_000);
    assert_eq!(fee_rate(fee, vsize), Some(5_000.0 / 140.0));
}

#[test]
fn user_total_lower_than_output_sum_reports_negative_fee_without_panicking() {
    let fee = fee_from_user_total(90_000, 100_000);
    assert_eq!(fee, -10_000);
    assert_eq!(fee_rate(fee, 100), Some(-100.0));
    assert_eq!(
        compare_to_floor(fee_rate(fee, 100), 4.0),
        FloorComparison::Below
    );
}

#[test]
fn floor_comparison_at_just_below_and_just_above() {
    let floor = 4.0;
    assert_eq!(
        compare_to_floor(Some(4.0), floor),
        FloorComparison::AtOrAbove
    );
    assert_eq!(
        compare_to_floor(Some(3.999_999_999), floor),
        FloorComparison::Below
    );
    assert_eq!(
        compare_to_floor(Some(4.000_000_001), floor),
        FloorComparison::AtOrAbove
    );
    assert_eq!(compare_to_floor(None, floor), FloorComparison::Unknown);
}

#[test]
fn fee_rate_is_unknown_for_zero_vsize() {
    assert_eq!(fee_rate(1_000, 0), None);
    assert_eq!(
        compare_to_floor(fee_rate(1_000, 0), 4.0),
        FloorComparison::Unknown
    );
}

#[test]
fn psbt_fee_reports_negative_fee_when_outputs_exceed_inputs_without_panicking() {
    let psbt = psbt(&[Utxo::Witness(1_000)], &[5_000]);
    assert_eq!(psbt_fee(&psbt), Ok(PsbtFee::Known { fee_sats: -4_000 }));
}

#[test]
fn input_sum_reports_overflow_instead_of_an_exact_value() {
    let psbt = psbt(&[Utxo::Witness(u64::MAX), Utxo::Witness(u64::MAX)], &[1]);
    assert_eq!(
        psbt_input_sum(&psbt),
        Err(PsbtAnalysisError::AmountOverflow)
    );
}

#[test]
fn output_sum_reports_overflow_instead_of_an_exact_value() {
    let tx = Transaction {
        version: Version::TWO,
        lock_time: LockTime::ZERO,
        input: vec![],
        output: vec![
            TxOut {
                value: Amount::from_sat(u64::MAX),
                script_pubkey: ScriptBuf::new(),
            },
            TxOut {
                value: Amount::from_sat(u64::MAX),
                script_pubkey: ScriptBuf::new(),
            },
        ],
    };
    assert_eq!(output_sum(&tx), Err(AmountOverflowError));
}

#[test]
fn psbt_map_counts_are_validated() {
    let mut input_mismatch = psbt(&[Utxo::Witness(1_000)], &[900]);
    input_mismatch.inputs.clear();
    assert_eq!(
        psbt_input_sum(&input_mismatch),
        Err(PsbtAnalysisError::InvalidPsbt(
            PsbtValidationError::InputCountMismatch {
                transaction: 1,
                maps: 0,
            }
        ))
    );

    let mut output_mismatch = psbt(&[Utxo::Witness(1_000)], &[900]);
    output_mismatch.outputs.clear();
    assert_eq!(
        psbt_output_sum(&output_mismatch),
        Err(PsbtAnalysisError::InvalidPsbt(
            PsbtValidationError::OutputCountMismatch {
                transaction: 1,
                maps: 0,
            }
        ))
    );
}

#[test]
fn non_witness_utxo_txid_and_vout_are_validated() {
    let mut wrong_txid = psbt(&[Utxo::NonWitness(1_000)], &[900]);
    wrong_txid.unsigned_tx.input[0].previous_output.txid = Txid::all_zeros();
    assert_eq!(
        psbt_input_sum(&wrong_txid),
        Err(PsbtAnalysisError::InvalidPsbt(
            PsbtValidationError::NonWitnessTxidMismatch { input: 0 }
        ))
    );

    let mut bad_vout = psbt(&[Utxo::NonWitness(1_000)], &[900]);
    bad_vout.unsigned_tx.input[0].previous_output.vout = 1;
    assert_eq!(
        psbt_input_sum(&bad_vout),
        Err(PsbtAnalysisError::InvalidPsbt(
            PsbtValidationError::NonWitnessVoutOutOfBounds {
                input: 0,
                vout: 1,
                outputs: 1,
            }
        ))
    );
}

#[test]
fn witness_and_non_witness_utxos_must_match() {
    let mut psbt = psbt(&[Utxo::NonWitness(1_000)], &[900]);
    psbt.inputs[0].witness_utxo = Some(TxOut {
        value: Amount::from_sat(999),
        script_pubkey: ScriptBuf::new(),
    });
    assert_eq!(
        psbt_fee(&psbt),
        Err(PsbtAnalysisError::InvalidPsbt(
            PsbtValidationError::WitnessUtxoMismatch { input: 0 }
        ))
    );
}

#[test]
fn vsize_and_fee_rate_compose_end_to_end_on_a_finalized_transaction() {
    use std::collections::BTreeMap;

    use bitcoin::PrivateKey;
    use bitcoin::PublicKey;
    use bitcoin::bip32::{DerivationPath, Fingerprint};
    use bitcoin::secp256k1::{Secp256k1, SecretKey};

    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_slice(&[7; 32]).expect("valid secret key");
    let private_key = PrivateKey::new(secret_key, bitcoin::Network::Bitcoin);
    let public_key: PublicKey = private_key.public_key(&secp);
    let script_pubkey = ScriptBuf::new_p2wpkh(&public_key.wpubkey_hash().expect("compressed key"));

    let unsigned_tx = Transaction {
        version: Version::TWO,
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint::new(Txid::all_zeros(), 0),
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: bitcoin::Witness::new(),
        }],
        output: vec![TxOut {
            value: Amount::from_sat(90_000),
            script_pubkey: script_pubkey.clone(),
        }],
    };

    let mut psbt = Psbt::from_unsigned_tx(unsigned_tx).expect("valid unsigned tx");
    psbt.inputs = vec![Input {
        witness_utxo: Some(TxOut {
            value: Amount::from_sat(100_000),
            script_pubkey,
        }),
        bip32_derivation: {
            let mut map = BTreeMap::new();
            map.insert(
                public_key.inner,
                (Fingerprint::default(), DerivationPath::default()),
            );
            map
        },
        ..Default::default()
    }];

    let fee = match psbt_fee(&psbt).expect("valid PSBT") {
        PsbtFee::Known { fee_sats } => fee_sats,
        PsbtFee::Unknown { .. } => panic!("every input carries witness_utxo"),
    };
    assert_eq!(fee, 10_000);

    let mut keymap: BTreeMap<PublicKey, PrivateKey> = BTreeMap::new();
    keymap.insert(public_key, private_key);
    psbt.sign(&keymap, &secp).expect("signs cleanly");

    let tx = match finalize(psbt).expect("complete PSBT finalizes") {
        FinalizedPsbt::Validated(tx) => tx,
        FinalizedPsbt::Unchecked { .. } => panic!("fixture has every UTXO"),
    };
    let vsize = vsize(&tx);
    assert!(vsize > 0);

    let rate = fee_rate(fee, vsize).expect("nonzero vsize");
    assert_eq!(rate, fee as f64 / vsize as f64);
}
