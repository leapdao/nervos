use super::*;
use ckb_testtool::context::Context;
use ckb_tool::ckb_types::{bytes::Bytes, core::TransactionBuilder, packed::*, prelude::*};
use ckb_tool::{ckb_error::assert_error_eq, ckb_script::{TransactionScriptError, ScriptError}};
use elliptic_curve::sec1::ToEncodedPoint;
use elliptic_curve::FieldBytes;
// use signature::signer::DigestSigner;
use hex::FromHex;
use k256::{
    ecdsa::{
        recoverable,
        signature::{Signature, Signer, DigestSigner},
        SigningKey, VerifyKey,
    },
    Secp256k1,
    
};
use rand::Rng;
use rand_core::OsRng;
use sha3::{Digest, Keccak256, Keccak256Full}; // requires 'getrandom' feature
use tiny_keccak::Keccak;
use tiny_keccak::Hasher;
use secp256k1::{Message, SecretKey, PublicKey, sign, Signature as Sig, RecoveryId};

const MAX_CYCLES: u64 = 100_000_000;

fn get_val_keys() -> (SigningKey, VerifyKey) {
    let signing_key = SigningKey::random(&mut OsRng); // Serialize with `::to_bytes()`
    let verify_key = signing_key.verify_key();
    (signing_key, verify_key)
}

fn gen_receipt(amount: u64, lock_hash: [u8; 32], tx_hash: [u8; 32]) -> [u8; 128] {
    let mut receipt: [u8; 128] = [0; 128];

    for (dst, src) in receipt[56..64].iter_mut().zip(&amount.to_be_bytes()) {
        *dst = *src
    }

    for (dst, src) in receipt[64..96].iter_mut().zip(&lock_hash) {
        *dst = *src
    }

    for (dst, src) in receipt[96..].iter_mut().zip(&tx_hash) {
        *dst = *src
    }

    receipt
}

// fn sign_receipt(receipt: [u8; 128], priv_key: SigningKey) -> recoverable::Signature {
//     let preamble: &[u8] = b"\x19Ethereum Signed Message:\n128";
//     let msg: Vec<u8> = [preamble, &receipt[..]].concat();
//     priv_key.sign(msg.as_slice())
// }

fn sign_receipt(receipt: [u8; 128], priv_key: SigningKey) -> recoverable::Signature {
    let preamble: &[u8] = b"\x19Ethereum Signed Message:\n128";
    println!("pramble {:?}", preamble);
    let msg: Vec<u8> = [preamble, &receipt[..]].concat();
    let mut digest = Keccak256::new();
    digest.update(msg.as_slice());
    println!("Receipt hash {:?}", hex::encode(digest.clone().finalize()));
    // println!("Signing bytes {:?}", msg.clone().as_slice());
    priv_key.sign_digest(digest)
}

fn sign_receipt2(receipt: [u8; 128], priv_key: SecretKey) -> (Sig, RecoveryId) {
    let preamble: &[u8] = b"\x19Ethereum Signed Message:\n128";
    let msg: Vec<u8> = [preamble, &receipt[..]].concat();
    let digest: [u8; 32] = Keccak256::digest(&msg[..]).into();
    
    println!("Receipt hash {:?}", hex::encode(&digest[..]));
    let message = Message::parse(&digest);
    sign(&message, &priv_key)
}

struct PayoutTestParams<'a> {
    audit_delay_trustee_lock_hash: [u8; 32],
    audit_delay_owner_lock_hash: [u8; 32],
    audit_delay_payout_amount: u64,
    bridge_state_id: [u8; 36],
    bridge_validators: Vec<&'a [u8]>,
    bridge_trustee: [u8; 32],
    sig: recoverable::Signature,
    receipt: [u8; 128],
    bridge_after_capacity: u64,
    bridge_before_capacity: u64,
    change_capacity: u64,
    bridge_data_before: Bytes,
    bridge_data_after: Bytes,
    funding_amount: u64,
    timeout: u64,
    error: Option<TransactionScriptError>,
}

fn test_payout(params: PayoutTestParams) {
    let mut context = Context::default();

    // load binaries
    let always_success_bin: Bytes = Loader::default().load_binary("anyone-can-spend");
    let bridge_bin: Bytes = Loader::default().load_binary("parent-bridge");
    let audit_delay_bin: Bytes = Loader::default().load_binary("audit-delay");

    // deploy binaries to cells
    let always_success_out_point = context.deploy_cell(always_success_bin);
    let bridge_out_point = context.deploy_cell(bridge_bin);
    let audit_delay_out_point = context.deploy_cell(audit_delay_bin);

    // make dep objects for our contracts
    let always_success_dep = CellDep::new_builder()
        .out_point(always_success_out_point.clone())
        .build();
    let bridge_dep = CellDep::new_builder()
        .out_point(bridge_out_point.clone())
        .build();
    let audit_delay_dep = CellDep::new_builder()
        .out_point(audit_delay_out_point.clone())
        .build();

    // always success args
    let always_success_args = Bytes::default();
    // audit delay args
    let audit_delay_args = Bytes::from(
        [
            &params.audit_delay_trustee_lock_hash[..],
            &params.audit_delay_owner_lock_hash[..],
            &params.timeout.to_be_bytes()[..],
        ]
        .concat(),
    );
    // bridge args
    let flat_validators = params
        .bridge_validators
        .iter()
        .map(|val| val.to_vec())
        .flatten()
        .collect::<Vec<u8>>();
    let bridge_args = Bytes::from(
        [
            &params.bridge_state_id[..],
            &params.bridge_trustee[..],
            flat_validators.as_slice(),
        ]
        .concat(),
    );

    // make our script objects
    let always_success_script = context
        .build_script(&always_success_out_point, always_success_args)
        .expect("script");
    let bridge_script = context
        .build_script(&bridge_out_point, bridge_args)
        .expect("script");
    let audit_delay_script = context
        .build_script(&audit_delay_out_point, audit_delay_args)
        .expect("script");

    // bridge witness
    let action_byte = Bytes::from(Vec::from_hex("00").unwrap());
    let signature = Bytes::from(Vec::from(params.sig.as_bytes()));
    
    let bridge_witness = Bytes::from(
        [
            action_byte,
            Bytes::from(Vec::from(&params.receipt[..])),
            signature,
        ]
        .concat(),
    );

    // input outpoints
    let prev_bridge_output = CellOutput::new_builder()
        .capacity(params.bridge_before_capacity.pack())
        .lock(always_success_script.clone())
        .type_(Some(bridge_script.clone()).pack())
        .build();
    let funding_output = CellOutput::new_builder()
        .capacity(params.funding_amount.pack())
        .lock(always_success_script.clone())
        .build();

    let prev_bridge_outpoint = context.create_cell(prev_bridge_output, params.bridge_data_before);
    let funding_outpoint = context.create_cell(funding_output, Bytes::default());

    let inputs = vec![
        // bridge input
        CellInput::new_builder()
            .previous_output(prev_bridge_outpoint)
            .build(),
        // funding input
        CellInput::new_builder()
            .previous_output(funding_outpoint)
            .build(),
    ];

    let outputs = vec![
        // bridge output
        CellOutput::new_builder()
            .capacity(params.bridge_after_capacity.pack())
            .lock(always_success_script.clone())
            .type_(Some(bridge_script.clone()).pack())
            .build(),
        // payment output
        CellOutput::new_builder()
            .capacity(params.audit_delay_payout_amount.pack())
            .lock(audit_delay_script.clone())
            .build(),
        // change output
        CellOutput::new_builder()
            .capacity(params.change_capacity.pack())
            .lock(always_success_script.clone())
            .build(),
    ];

    let witnesses = vec![bridge_witness, Bytes::new(), Bytes::new()];

    let outputs_data = vec![
        params.bridge_data_after,
        // Bytes::from(Vec::from(&Keccak256::digest(&params.receipt[..])[..])),
        Bytes::new(),
        Bytes::new(),
    ];

    // build transaction
    let tx = TransactionBuilder::default()
        .inputs(inputs)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(audit_delay_dep)
        .cell_dep(bridge_dep)
        .cell_dep(always_success_dep)
        .witnesses(witnesses.pack())
        .build();
    let tx = context.complete_tx(tx);

    // run
    match params.error {
        None => {
            context
                .verify_tx(&tx, MAX_CYCLES)
                .expect("pass verification");
        }
        Some(error) => {
            let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();
            assert_error_eq!(err, error);
        }
    }
}

#[test]
fn test_unlock() {
        
    let payout_amount: u64 = 32 * 100000000 * 100;
    let mut receipt_owner_lock_hash = [0u8; 32];
    hex::decode_to_slice("0efdcdec4f8490c951e4a225db3bce7274278b5d05be24d7f692454488412ad7", &mut receipt_owner_lock_hash as &mut [u8]);
    let mut receipt_tx_hash = [0u8; 32];
    hex::decode_to_slice("0000000000000000000000000000000000000000000000000000000000000000", &mut receipt_tx_hash as &mut [u8]);
    
    let receipt = gen_receipt(payout_amount, receipt_owner_lock_hash, receipt_tx_hash);
    let sig: recoverable::Signature = sign_receipt(receipt, priv_key);

    let validator_address =
        &Keccak256::digest(&pub_key.to_encoded_point(false).as_bytes()[1..65])[12..];
    let validator_list = vec![validator_address];

    let params = PayoutTestParams {
        audit_delay_trustee_lock_hash: trustee_lock_hash,
        audit_delay_owner_lock_hash: receipt_owner_lock_hash,
        audit_delay_payout_amount: payout_amount,
        bridge_state_id: bridge_state_id,
        bridge_validators: validator_list,
        bridge_trustee: trustee_lock_hash,
        sig: sig,
        receipt: receipt,
        bridge_after_capacity: 90,
        bridge_before_capacity: 100,
        change_capacity: 8,
        bridge_data_before: Bytes::default(),
        bridge_data_after: Bytes::from(Vec::from(&Keccak256::digest(&receipt[..])[..])),
        funding_amount: 10,
        timeout: 100,
        error: None,
    };

    test_payout(params);
}

#[test]
fn test_unknown_receipt_signer() {
    let trustee_lock_hash = rand::thread_rng().gen::<[u8; 32]>();
    let bridge_state_id = [0u8; 36];

    let payout_amount = 10;

    let (priv_key, pub_key) = get_val_keys();
    let (dummy_priv, dummy_pub) = get_val_keys();
    let receipt_owner_lock_hash = rand::thread_rng().gen::<[u8; 32]>();
    let receipt_tx_hash = rand::thread_rng().gen::<[u8; 32]>();
    let receipt = gen_receipt(payout_amount, receipt_owner_lock_hash, receipt_tx_hash);
    let sig: recoverable::Signature = sign_receipt(receipt, priv_key);

    let validator_address =
        &Keccak256::digest(&dummy_pub.to_encoded_point(false).as_bytes()[1..65])[12..];
    let validator_list = vec![validator_address];

    let params = PayoutTestParams {
        audit_delay_trustee_lock_hash: trustee_lock_hash,
        audit_delay_owner_lock_hash: receipt_owner_lock_hash,
        audit_delay_payout_amount: payout_amount,
        bridge_state_id: bridge_state_id,
        bridge_validators: validator_list,
        bridge_trustee: trustee_lock_hash,
        sig: sig,
        receipt: receipt,
        bridge_after_capacity: 90,
        bridge_before_capacity: 100,
        change_capacity: 8,
        bridge_data_before: Bytes::default(),
        bridge_data_after: Bytes::from(Vec::from(&Keccak256::digest(&receipt[..])[..])),
        funding_amount: 10,
        timeout: 100,
        error: Some(ScriptError::ValidationFailure(21).input_type_script(0)),
    };

    test_payout(params);
}

#[test]
fn test_signature_quorum_not_met() {
    let trustee_lock_hash = rand::thread_rng().gen::<[u8; 32]>();
    let bridge_state_id = [0u8; 36];

    let payout_amount = 10;

    let (priv_key, pub_key) = get_val_keys();
    let (priv_key1, pub_key1) = get_val_keys();
    let (priv_key2, pub_key2) = get_val_keys();
    let receipt_owner_lock_hash = rand::thread_rng().gen::<[u8; 32]>();
    let receipt_tx_hash = rand::thread_rng().gen::<[u8; 32]>();
    let receipt = gen_receipt(payout_amount, receipt_owner_lock_hash, receipt_tx_hash);
    let sig: recoverable::Signature = sign_receipt(receipt, priv_key);

    let validator_address =
        &Keccak256::digest(&pub_key.to_encoded_point(false).as_bytes()[1..65])[12..];
    let validator_address1 =
        &Keccak256::digest(&pub_key1.to_encoded_point(false).as_bytes()[1..65])[12..];
    let validator_address2 =
        &Keccak256::digest(&pub_key2.to_encoded_point(false).as_bytes()[1..65])[12..];
    let validator_list = vec![validator_address, validator_address1, validator_address2];

    let params = PayoutTestParams {
        audit_delay_trustee_lock_hash: trustee_lock_hash,
        audit_delay_owner_lock_hash: receipt_owner_lock_hash,
        audit_delay_payout_amount: payout_amount,
        bridge_state_id: bridge_state_id,
        bridge_validators: validator_list,
        bridge_trustee: trustee_lock_hash,
        sig: sig,
        receipt: receipt,
        bridge_after_capacity: 90,
        bridge_before_capacity: 100,
        change_capacity: 8,
        bridge_data_before: Bytes::default(),
        bridge_data_after: Bytes::from(Vec::from(&Keccak256::digest(&receipt[..])[..])),
        funding_amount: 10,
        timeout: 100,
        error: Some(ScriptError::ValidationFailure(22).input_type_script(0)),
    };

    test_payout(params);
}

#[test]
fn test_invalid_withdrawal_capacity() {
    let trustee_lock_hash = rand::thread_rng().gen::<[u8; 32]>();
    let bridge_state_id = [0u8; 36];

    println!("{:?}", hex::encode(&receipt[..]));

    let mut priv_bytes = [0u8; 32];
    hex::decode_to_slice("278a5de700e29faae8e40e366ec5012b5ec63d36ec77e8a2417154cc1d25383f", &mut priv_bytes as &mut [u8]);
    let signing_key = SigningKey::new(&priv_bytes[..]).unwrap();
    let verify_key = signing_key.verify_key();
    let validator_address =
        &Keccak256::digest(&pub_key.to_encoded_point(false).as_bytes()[1..65])[12..];
    let validator_list = vec![validator_address];

    let params = PayoutTestParams {
        audit_delay_trustee_lock_hash: trustee_lock_hash,
        audit_delay_owner_lock_hash: receipt_owner_lock_hash,
        audit_delay_payout_amount: payout_amount,
        bridge_state_id: bridge_state_id,
        bridge_validators: validator_list,
        bridge_trustee: trustee_lock_hash,
        sig: sig,
        receipt: receipt,
        bridge_after_capacity: 95,
        bridge_before_capacity: 100,
        change_capacity: 8,
        bridge_data_before: Bytes::default(),
        bridge_data_after: Bytes::from(Vec::from(&Keccak256::digest(&receipt[..])[..])),
        funding_amount: 10,
        timeout: 100,
        error: Some(ScriptError::ValidationFailure(23).input_type_script(0)),
    };

    test_payout(params);
}

#[test]
fn test_data_updated_incorrectly() {
    let trustee_lock_hash = rand::thread_rng().gen::<[u8; 32]>();
    let bridge_state_id = [0u8; 36];

    let payout_amount = 10;

    let (priv_key, pub_key) = get_val_keys();
    let receipt_owner_lock_hash = rand::thread_rng().gen::<[u8; 32]>();
    let receipt_tx_hash = rand::thread_rng().gen::<[u8; 32]>();
    let receipt = gen_receipt(payout_amount, receipt_owner_lock_hash, receipt_tx_hash);
    let sig: recoverable::Signature = sign_receipt(receipt, priv_key);

    let validator_address =
        &Keccak256::digest(&pub_key.to_encoded_point(false).as_bytes()[1..65])[12..];
    let validator_list = vec![validator_address];

    let params = PayoutTestParams {
        audit_delay_trustee_lock_hash: trustee_lock_hash,
        audit_delay_owner_lock_hash: receipt_owner_lock_hash,
        audit_delay_payout_amount: payout_amount,
        bridge_state_id: bridge_state_id,
        bridge_validators: validator_list,
        bridge_trustee: trustee_lock_hash,
        sig: sig,
        receipt: receipt,
        bridge_after_capacity: 90,
        bridge_before_capacity: 100,
        change_capacity: 8,
        bridge_data_before: Bytes::default(),
        bridge_data_after: Bytes::new(),
        funding_amount: 10,
        timeout: 100,
        error: Some(ScriptError::ValidationFailure(24).input_type_script(0)),
    };

    test_payout(params);
}

#[test]
fn test_wrong_trustee_in_payout() {
    let trustee_lock_hash = rand::thread_rng().gen::<[u8; 32]>();
    let bridge_state_id = [0u8; 36];

    let payout_amount = 10;

    let (priv_key, pub_key) = get_val_keys();
    let receipt_owner_lock_hash = rand::thread_rng().gen::<[u8; 32]>();
    let receipt_tx_hash = rand::thread_rng().gen::<[u8; 32]>();
    let receipt = gen_receipt(payout_amount, receipt_owner_lock_hash, receipt_tx_hash);
    let sig: recoverable::Signature = sign_receipt(receipt, priv_key);

    let validator_address =
        &Keccak256::digest(&pub_key.to_encoded_point(false).as_bytes()[1..65])[12..];
    let validator_list = vec![validator_address];

    let params = PayoutTestParams {
        audit_delay_trustee_lock_hash: trustee_lock_hash,
        audit_delay_owner_lock_hash: receipt_owner_lock_hash,
        audit_delay_payout_amount: payout_amount,
        bridge_state_id: bridge_state_id,
        bridge_validators: validator_list,
        bridge_trustee: receipt_owner_lock_hash,
        sig: sig,
        receipt: receipt,
        bridge_after_capacity: 90,
        bridge_before_capacity: 100,
        change_capacity: 8,
        bridge_data_before: Bytes::default(),
        bridge_data_after: Bytes::from(Vec::from(&Keccak256::digest(&receipt[..])[..])),
        funding_amount: 10,
        timeout: 100,
        error: Some(ScriptError::ValidationFailure(25).input_type_script(0)),
    };

    test_payout(params);
}

#[test]
fn test_wrong_payout_destination() {
    let trustee_lock_hash = rand::thread_rng().gen::<[u8; 32]>();
    let bridge_state_id = [0u8; 36];

    let payout_amount = 10;

    let (priv_key, pub_key) = get_val_keys();
    let receipt_owner_lock_hash = rand::thread_rng().gen::<[u8; 32]>();
    let receipt_tx_hash = rand::thread_rng().gen::<[u8; 32]>();
    let receipt = gen_receipt(payout_amount, trustee_lock_hash, receipt_tx_hash);
    let sig: recoverable::Signature = sign_receipt(receipt, priv_key);

    let validator_address =
        &Keccak256::digest(&pub_key.to_encoded_point(false).as_bytes()[1..65])[12..];
    let validator_list = vec![validator_address];

    let params = PayoutTestParams {
        audit_delay_trustee_lock_hash: trustee_lock_hash,
        audit_delay_owner_lock_hash: receipt_owner_lock_hash,
        audit_delay_payout_amount: payout_amount,
        bridge_state_id: bridge_state_id,
        bridge_validators: validator_list,
        bridge_trustee: trustee_lock_hash,
        sig: sig,
        receipt: receipt,
        bridge_after_capacity: 90,
        bridge_before_capacity: 100,
        change_capacity: 8,
        bridge_data_before: Bytes::default(),
        bridge_data_after: Bytes::from(Vec::from(&Keccak256::digest(&receipt[..])[..])),
        funding_amount: 10,
        timeout: 100,
        error: Some(ScriptError::ValidationFailure(26).input_type_script(0)),
    };

    test_payout(params);
}

#[test]
fn test_wrong_timeout() {
    let trustee_lock_hash = rand::thread_rng().gen::<[u8; 32]>();
    let bridge_state_id = [0u8; 36];

    let payout_amount = 10;

    let (priv_key, pub_key) = get_val_keys();
    let receipt_owner_lock_hash = rand::thread_rng().gen::<[u8; 32]>();
    let receipt_tx_hash = rand::thread_rng().gen::<[u8; 32]>();
    let receipt = gen_receipt(payout_amount, receipt_owner_lock_hash, receipt_tx_hash);
    let sig: recoverable::Signature = sign_receipt(receipt, priv_key);

    let validator_address =
        &Keccak256::digest(&pub_key.to_encoded_point(false).as_bytes()[1..65])[12..];
    let validator_list = vec![validator_address];

    let params = PayoutTestParams {
        audit_delay_trustee_lock_hash: trustee_lock_hash,
        audit_delay_owner_lock_hash: receipt_owner_lock_hash,
        audit_delay_payout_amount: payout_amount,
        bridge_state_id: bridge_state_id,
        bridge_validators: validator_list,
        bridge_trustee: trustee_lock_hash,
        sig: sig,
        receipt: receipt,
        bridge_after_capacity: 90,
        bridge_before_capacity: 100,
        change_capacity: 8,
        bridge_data_before: Bytes::default(),
        bridge_data_after: Bytes::from(Vec::from(&Keccak256::digest(&receipt[..])[..])),
        funding_amount: 10,
        timeout: 101,
        error: Some(ScriptError::ValidationFailure(27).input_type_script(0)),
    };

    test_payout(params);
}

#[test]
fn test_receipt_already_used() {
    let trustee_lock_hash = rand::thread_rng().gen::<[u8; 32]>();
    let bridge_state_id = [0u8; 36];

    let payout_amount = 10;

    let (priv_key, pub_key) = get_val_keys();
    let receipt_owner_lock_hash = rand::thread_rng().gen::<[u8; 32]>();
    let receipt_tx_hash = rand::thread_rng().gen::<[u8; 32]>();
    let receipt = gen_receipt(payout_amount, receipt_owner_lock_hash, receipt_tx_hash);
    let sig: recoverable::Signature = sign_receipt(receipt, priv_key);

    let validator_address =
        &Keccak256::digest(&pub_key.to_encoded_point(false).as_bytes()[1..65])[12..];
    let validator_list = vec![validator_address];

    let params = PayoutTestParams {
        audit_delay_trustee_lock_hash: trustee_lock_hash,
        audit_delay_owner_lock_hash: receipt_owner_lock_hash,
        audit_delay_payout_amount: payout_amount,
        bridge_state_id: bridge_state_id,
        bridge_validators: validator_list,
        bridge_trustee: trustee_lock_hash,
        sig: sig,
        receipt: receipt,
        bridge_after_capacity: 90,
        bridge_before_capacity: 100,
        change_capacity: 8,
        bridge_data_before: Bytes::from(Vec::from(&Keccak256::digest(&receipt[..])[..])),
        bridge_data_after: Bytes::from([&Keccak256::digest(&receipt[..])[..], &Keccak256::digest(&receipt[..])[..]].concat()),
        funding_amount: 10,
        timeout: 100,
        error: Some(ScriptError::ValidationFailure(28).input_type_script(0)),
    };

    test_payout(params);
}
 
