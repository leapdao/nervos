use super::*;
use ckb_testtool::{builtin::ALWAYS_SUCCESS, context::Context};
use ckb_tool::ckb_types::{bytes::Bytes, core::TransactionBuilder, packed::*, prelude::*};
use hex::FromHex;
use ckb_tool::{ckb_error::assert_error_eq, ckb_script::ScriptError};

const MAX_CYCLES: u64 = 10_000_000;


#[test]
fn test_success() {
    let mut context = Context::default();

    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());
    let lock_script = context
        .build_script(&always_success_out_point, Default::default())
        .expect("script");
    let lock_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point)
        .build();

    let validator_list = Bytes::from(Vec::from_hex("1122334411223343241123344112233441122344112233441122334411223344000000000000000000000000112233445566778899001122334455667788990000000000000000000000000000000000000000000000000000000000000004D2AAAAAAAA").unwrap());

    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(10u64.pack())
            .lock(lock_script.clone())
            .build(),
        Default::default(),
    );
    let input = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();

    let tx_hash: &[u8] = &*input.previous_output().tx_hash().raw_data();
    let index: &[u8] = &*input.previous_output().index().raw_data();
    let state_id = Bytes::from([tx_hash, index].concat());

    let type_script_args = Bytes::from([&*state_id, &*validator_list].concat());

    let contract_bin: Bytes = Loader::default().load_binary("parent-bridge");
    let contract_out_point = context.deploy_cell(contract_bin);
    let bridge_script = context
        .build_script(&contract_out_point, type_script_args)
        .expect("script");
    let bridge_script_dep = CellDep::new_builder().out_point(contract_out_point).build();

    let outputs = vec![CellOutput::new_builder()
        .capacity(0u64.pack())
        .lock(lock_script.clone())
        .type_(Some(bridge_script.clone()).pack())
        .build()];

    let outputs_data = vec![Bytes::new(); 1];

    // in combat the secp256 lock script would check the withness
    // for a signature, hence we can't use an actionByte here.
    let witnesses = vec![Bytes::new()];

    // build transaction
    let tx = TransactionBuilder::default()
        .input(input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(lock_script_dep)
        .cell_dep(bridge_script_dep)
        .witnesses(witnesses.pack())
        .build();
    let tx = context.complete_tx(tx);

    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}

// #[test]
// fn test_state_transition_does_not_exist() {
//     let mut context = Context::default();

//     let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());
//     let lock_script = context
//         .build_script(&always_success_out_point, Default::default())
//         .expect("script");
//     let lock_script_dep = CellDep::new_builder()
//         .out_point(always_success_out_point)
//         .build();

//     let validator_list = Bytes::from(Vec::from_hex("1122334411223343241123344112233441122344112233441122334411223344000000000000000000000000112233445566778899001122334455667788990000000000000000000000000000000000000000000000000000000000000004D2AAAAAAAA").unwrap());

//     let input_out_point = context.create_cell(
//         CellOutput::new_builder()
//             .capacity(10u64.pack())
//             .lock(lock_script.clone())
//             .build(),
//         Default::default(),
//     );
//     let input = CellInput::new_builder()
//         .previous_output(input_out_point)
//         .build();

//     let tx_hash: &[u8] = &*input.previous_output().tx_hash().raw_data();
//     let index: &[u8] = &*input.previous_output().index().raw_data();
//     let state_id = Bytes::from([tx_hash, index].concat());

//     let type_script_args = Bytes::from([&*state_id, &*validator_list].concat());

//     let contract_bin: Bytes = Loader::default().load_binary("parent-bridge");
//     let contract_out_point = context.deploy_cell(contract_bin);
//     let bridge_script = context
//         .build_script(&contract_out_point, type_script_args)
//         .expect("script");
//     let bridge_script_dep = CellDep::new_builder().out_point(contract_out_point).build();

//     let outputs = vec![CellOutput::new_builder()
//         .capacity(0u64.pack())
//         .lock(lock_script.clone())
//         .type_(Some(bridge_script.clone()).pack())
//         .build()];

//     let outputs_data = vec![Bytes::new(); 1];

//     let witnesses = vec![Bytes::from(vec![18 as u8; 1]); 1];

//     let tx = TransactionBuilder::default()
//         .input(input)
//         .outputs(outputs)
//         .outputs_data(outputs_data.pack())
//         .cell_dep(lock_script_dep)
//         .cell_dep(bridge_script_dep)
//         .witnesses(witnesses.pack())
//         .build();
//     let tx = context.complete_tx(tx);

//     let err = context
//         .verify_tx(&tx, MAX_CYCLES)
//         .unwrap_err();

//     assert_error_eq!(err, ScriptError::ValidationFailure(5));
// }

#[test]
fn test_wrong_validator_list_length() {
    let mut context = Context::default();

    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());
    let lock_script = context
        .build_script(&always_success_out_point, Default::default())
        .expect("script");
    let lock_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point)
        .build();

    let validator_list = Bytes::from(Vec::from_hex("112233441122411223344112233441122334411223344112233441122334400000000000000000000000011223344556677889900112233445566778899000000000000000000000000000000000000000000000000000000000000004D2AAAAAAAA").unwrap());

    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(10u64.pack())
            .lock(lock_script.clone())
            .build(),
        Default::default(),
    );
    let input = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();

    let tx_hash: &[u8] = &*input.previous_output().tx_hash().raw_data();
    let index: &[u8] = &*input.previous_output().index().raw_data();
    let state_id = Bytes::from([tx_hash, index].concat());

    let type_script_args = Bytes::from([&*state_id, &*validator_list].concat());

    let contract_bin: Bytes = Loader::default().load_binary("parent-bridge");
    let contract_out_point = context.deploy_cell(contract_bin);
    let bridge_script = context
        .build_script(&contract_out_point, type_script_args)
        .expect("script");
    let bridge_script_dep = CellDep::new_builder().out_point(contract_out_point).build();

    let outputs = vec![CellOutput::new_builder()
        .capacity(0u64.pack())
        .lock(lock_script.clone())
        .type_(Some(bridge_script.clone()).pack())
        .build()];

    let outputs_data = vec![Bytes::new(); 1];

    let witnesses = vec![Bytes::from(vec![0 as u8; 1]); 1];

    let tx = TransactionBuilder::default()
        .input(input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(lock_script_dep)
        .cell_dep(bridge_script_dep)
        .witnesses(witnesses.pack())
        .build();
    let tx = context.complete_tx(tx);

    let err = context
        .verify_tx(&tx, MAX_CYCLES)
        .unwrap_err();

    assert_error_eq!(err, ScriptError::ValidationFailure(6));
}

#[test]
fn test_wrong_lock_script() {
    let mut context = Context::default();

    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());
    let lock_script = context
        .build_script(&always_success_out_point, Default::default())
        .expect("script");
    let lock_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point)
        .build();

    let validator_list = Bytes::from(Vec::from_hex("1122334411223343241123344112233441122344112233441122334411223344000000000000000000000000112233445566778899001122334455667788990000000000000000000000000000000000000000000000000000000000000004D2AAAAAAAA").unwrap());

    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(10u64.pack())
            .lock(lock_script.clone())
            .build(),
        Default::default(),
    );
    let input = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();

    let tx_hash: &[u8] = &*input.previous_output().tx_hash().raw_data();
    let index: &[u8] = &*input.previous_output().index().raw_data();
    let state_id = Bytes::from([tx_hash, index].concat());

    let type_script_args = Bytes::from([&*state_id, &*validator_list].concat());

    let contract_bin: Bytes = Loader::default().load_binary("parent-bridge");
    let contract_out_point = context.deploy_cell(contract_bin);
    let bridge_script = context
        .build_script(&contract_out_point, type_script_args)
        .expect("script");
    let bridge_script_dep = CellDep::new_builder().out_point(contract_out_point).build();

    let outputs = vec![CellOutput::new_builder()
        .capacity(0u64.pack())
        .lock(bridge_script.clone())
        .type_(Some(bridge_script.clone()).pack())
        .build()];

    let outputs_data = vec![Bytes::new(); 1];

    let witnesses = vec![Bytes::from(vec![0 as u8; 1]); 1];

    let tx = TransactionBuilder::default()
        .input(input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(lock_script_dep)
        .cell_dep(bridge_script_dep)
        .witnesses(witnesses.pack())
        .build();
    let tx = context.complete_tx(tx);

    let err = context
        .verify_tx(&tx, MAX_CYCLES)
        .unwrap_err();

    assert_error_eq!(err, ScriptError::ValidationFailure(7));
}

#[test]
fn test_wrong_type_script() {
    let mut context = Context::default();

    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());
    let lock_script = context
        .build_script(&always_success_out_point, Default::default())
        .expect("script");
    let lock_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point)
        .build();

    let validator_list = Bytes::from(Vec::from_hex("1122334411223343241123344112233441122344112233441122334411223344000000000000000000000000112233445566778899001122334455667788990000000000000000000000000000000000000000000000000000000000000004D2AAAAAAAA").unwrap());

    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(10u64.pack())
            .lock(lock_script.clone())
            .build(),
        Default::default(),
    );
      let input = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();

    let tx_hash: &[u8] = &*input.previous_output().tx_hash().raw_data();
    let index: &[u8] = &*input.previous_output().index().raw_data();
    let state_id = Bytes::from([tx_hash, index].concat());

    let type_script_args = Bytes::from([&*state_id, &*validator_list].concat());

    let contract_bin: Bytes = Loader::default().load_binary("parent-bridge");
    let contract_out_point = context.deploy_cell(contract_bin);
    let bridge_script = context
        .build_script(&contract_out_point, type_script_args)
        .expect("script");
    let bridge_script_dep = CellDep::new_builder().out_point(contract_out_point).build();

    let outputs = vec![CellOutput::new_builder()
        .capacity(0u64.pack())
        .lock(lock_script.clone())
        .type_(Some(lock_script.clone()).pack())
        .build(),
        CellOutput::new_builder()
        .capacity(0u64.pack())
        .lock(lock_script.clone())
        .type_(Some(bridge_script.clone()).pack())
        .build()];

    let outputs_data = vec![Bytes::new(); 2];

    let witnesses = vec![Bytes::from(vec![0 as u8; 1]); 1];

    let tx = TransactionBuilder::default()
        .input(input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(lock_script_dep)
        .cell_dep(bridge_script_dep)
        .witnesses(witnesses.pack())
        .build();
    let tx = context.complete_tx(tx);

    let err = context
        .verify_tx(&tx, MAX_CYCLES)
        .unwrap_err();

    assert_error_eq!(err, ScriptError::ValidationFailure(8));
}

#[test]
fn test_data_length_not_zero() {
    let mut context = Context::default();

    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());
    let lock_script = context
        .build_script(&always_success_out_point, Default::default())
        .expect("script");
    let lock_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point)
        .build();

    let validator_list = Bytes::from(Vec::from_hex("1122334411223343241123344112233441122344112233441122334411223344000000000000000000000000112233445566778899001122334455667788990000000000000000000000000000000000000000000000000000000000000004D2AAAAAAAA").unwrap());

    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(10u64.pack())
            .lock(lock_script.clone())
            .build(),
        Default::default(),
    );
      let input = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();

    let tx_hash: &[u8] = &*input.previous_output().tx_hash().raw_data();
    let index: &[u8] = &*input.previous_output().index().raw_data();
    let state_id = Bytes::from([tx_hash, index].concat());

    let type_script_args = Bytes::from([&*state_id, &*validator_list].concat());

    let contract_bin: Bytes = Loader::default().load_binary("parent-bridge");
    let contract_out_point = context.deploy_cell(contract_bin);
    let bridge_script = context
        .build_script(&contract_out_point, type_script_args)
        .expect("script");
    let bridge_script_dep = CellDep::new_builder().out_point(contract_out_point).build();

    let outputs = vec![CellOutput::new_builder()
        .capacity(0u64.pack())
        .lock(lock_script.clone())
        .type_(Some(bridge_script.clone()).pack())
        .build()];

    let outputs_data = vec![("af1872fa2").pack()];

    let witnesses = vec![Bytes::from(vec![0 as u8; 1]); 1];

    let tx = TransactionBuilder::default()
        .input(input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(lock_script_dep)
        .cell_dep(bridge_script_dep)
        .witnesses(witnesses.pack())
        .build();
    let tx = context.complete_tx(tx);

    let err = context
        .verify_tx(&tx, MAX_CYCLES)
        .unwrap_err();
    
    assert_error_eq!(err, ScriptError::ValidationFailure(9));
}

#[test]
fn test_wrong_state_id() {
    let mut context = Context::default();

    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());
    let lock_script = context
        .build_script(&always_success_out_point, Default::default())
        .expect("script");
    let lock_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point)
        .build();

    let validator_list = Bytes::from(Vec::from_hex("1122334411223343241123344112233441122344112233441122334411223344000000000000000000000000112233445566778899001122334455667788990000000000000000000000000000000000000000000000000000000000000004D2AAAAAAAA").unwrap());

    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(10u64.pack())
            .lock(lock_script.clone())
            .build(),
        Default::default(),
    );
      let input = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();

    let tx_hash: &[u8] = &*input.previous_output().tx_hash().raw_data();
    // let index: &[u8] = &*input.previous_output().index().raw_data();
    let scrambler: &[u8] = &[1, 1, 1, 1];
    let state_id = Bytes::from([tx_hash, scrambler].concat());

    let type_script_args = Bytes::from([&*state_id, &*validator_list].concat());

    let contract_bin: Bytes = Loader::default().load_binary("parent-bridge");
    let contract_out_point = context.deploy_cell(contract_bin);
    let bridge_script = context
        .build_script(&contract_out_point, type_script_args)
        .expect("script");
    let bridge_script_dep = CellDep::new_builder().out_point(contract_out_point).build();

    let outputs = vec![CellOutput::new_builder()
        .capacity(0u64.pack())
        .lock(lock_script.clone())
        .type_(Some(bridge_script.clone()).pack())
        .build()];

        let outputs_data = vec![Bytes::new(); 1];

    let witnesses = vec![Bytes::from(vec![0 as u8; 1]); 1];

    let tx = TransactionBuilder::default()
        .input(input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(lock_script_dep)
        .cell_dep(bridge_script_dep)
        .witnesses(witnesses.pack())
        .build();
    let tx = context.complete_tx(tx);

    let err = context
        .verify_tx(&tx, MAX_CYCLES)
        .unwrap_err();

    assert_error_eq!(err, ScriptError::ValidationFailure(10));
}

#[test]
fn test_too_many_type_outputs() {
    let mut context = Context::default();

    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());
    let lock_script = context
        .build_script(&always_success_out_point, Default::default())
        .expect("script");
    let lock_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point)
        .build();

    let validator_list = Bytes::from(Vec::from_hex("1122334411223343241123344112233441122344112233441122334411223344000000000000000000000000112233445566778899001122334455667788990000000000000000000000000000000000000000000000000000000000000004D2AAAAAAAA").unwrap());

    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(10u64.pack())
            .lock(lock_script.clone())
            .build(),
        Default::default(),
    );
      let input = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();

    let tx_hash: &[u8] = &*input.previous_output().tx_hash().raw_data();
    let index: &[u8] = &*input.previous_output().index().raw_data();
    let state_id = Bytes::from([tx_hash, index].concat());

    let type_script_args = Bytes::from([&*state_id, &*validator_list].concat());

    let contract_bin: Bytes = Loader::default().load_binary("parent-bridge");
    let contract_out_point = context.deploy_cell(contract_bin);
    let bridge_script = context
        .build_script(&contract_out_point, type_script_args)
        .expect("script");
    let bridge_script_dep = CellDep::new_builder().out_point(contract_out_point).build();

    let outputs = vec![CellOutput::new_builder()
        .capacity(0u64.pack())
        .lock(lock_script.clone())
        .type_(Some(bridge_script.clone()).pack())
        .build(),
        CellOutput::new_builder()
        .capacity(0u64.pack())
        .lock(lock_script.clone())
        .type_(Some(bridge_script.clone()).pack())
        .build()];

    let outputs_data = vec![Bytes::new(); 2];

    let witnesses = vec![Bytes::from(vec![0 as u8; 1]); 1];

    let tx = TransactionBuilder::default()
        .input(input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(lock_script_dep)
        .cell_dep(bridge_script_dep)
        .witnesses(witnesses.pack())
        .build();
    let tx = context.complete_tx(tx);

    let err = context
        .verify_tx(&tx, MAX_CYCLES)
        .unwrap_err();

    assert_error_eq!(err, ScriptError::ValidationFailure(11));
}

#[test]
fn test_deposit_with_lock() {
    let mut context = Context::default();

    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());
    let always_success_script = context
        .build_script(&always_success_out_point, Default::default())
        .expect("script");
    let always_success_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point)
        .build();

    let always_success_script_hash = always_success_script.calc_script_hash().raw_data();
    
    let deposit_lock_args : Bytes = Bytes::from([&*always_success_script_hash, &[0; 32]].concat());
    let deposit_lock_bin: Bytes = Loader::default().load_binary("deposit-lock");
    let deposit_lock_out_point = context.deploy_cell(deposit_lock_bin);
    let deposit_lock_script = context
        .build_script(&deposit_lock_out_point, deposit_lock_args)
        .expect("script");
    let deposit_lock_dep = CellDep::new_builder().out_point(deposit_lock_out_point).build();

    let input0_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(10u64.pack())
            .lock(always_success_script.clone())
            .build(),
        Default::default(),
    );
    let input0 = CellInput::new_builder()
        .previous_output(input0_out_point)
        .build();

    let input1_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(10u64.pack())
            .lock(deposit_lock_script.clone())
            .build(),
        Default::default(),
    );
    let input1 = CellInput::new_builder()
        .previous_output(input1_out_point)
        .build();

    let outputs = vec![CellOutput::new_builder()
        .capacity(20u64.pack())
        .lock(always_success_script.clone())
        .build()];
    let outputs_data = vec![Bytes::new()];

    // build transaction
    let tx = TransactionBuilder::default()
        .input(input0)
        .input(input1)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(always_success_script_dep)
        .cell_dep(deposit_lock_dep)
        .build();
    let tx = context.complete_tx(tx);

    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}

#[test]
fn test_deposit_with_type() {
    let mut context = Context::default();

    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());
    let always_success_script = context
        .build_script(&always_success_out_point, Default::default())
        .expect("script");
    let always_success_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point)
        .build();

    let always_success_script_hash = always_success_script.calc_script_hash().raw_data();
    
    let deposit_lock_args : Bytes = Bytes::from([&[0; 32], &*always_success_script_hash].concat());
    let deposit_lock_bin: Bytes = Loader::default().load_binary("deposit-lock");
    let deposit_lock_out_point = context.deploy_cell(deposit_lock_bin);
    let deposit_lock_script = context
        .build_script(&deposit_lock_out_point, deposit_lock_args)
        .expect("script");
    let deposit_lock_dep = CellDep::new_builder().out_point(deposit_lock_out_point).build();

    let input0_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(10u64.pack())
            .lock(always_success_script.clone())
            .type_(Some(always_success_script.clone()).pack())
            .build(),
        Default::default(),
    );
    let input0 = CellInput::new_builder()
        .previous_output(input0_out_point)
        .build();

    let input1_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(10u64.pack())
            .lock(deposit_lock_script.clone())
            .build(),
        Default::default(),
    );
    let input1 = CellInput::new_builder()
        .previous_output(input1_out_point)
        .build();

    let outputs = vec![CellOutput::new_builder()
        .capacity(20u64.pack())
        .lock(always_success_script.clone())
        .build()];
    let outputs_data = vec![Bytes::new()];

    // build transaction
    let tx = TransactionBuilder::default()
        .input(input0)
        .input(input1)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(always_success_script_dep)
        .cell_dep(deposit_lock_dep)
        .build();
    let tx = context.complete_tx(tx);

    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}

#[test]
fn test_deposit_wrong_scripts() {
    let mut context = Context::default();

    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());
    let always_success_script = context
        .build_script(&always_success_out_point, Default::default())
        .expect("script");
    let always_success_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point)
        .build();

    let deposit_lock_args : Bytes = Bytes::from([[0; 32], [0; 32]].concat());
    let deposit_lock_bin: Bytes = Loader::default().load_binary("deposit-lock");
    let deposit_lock_out_point = context.deploy_cell(deposit_lock_bin);
    let deposit_lock_script = context
        .build_script(&deposit_lock_out_point, deposit_lock_args)
        .expect("script");
    let deposit_lock_dep = CellDep::new_builder().out_point(deposit_lock_out_point).build();

    let input0_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(10u64.pack())
            .lock(always_success_script.clone())
            .type_(Some(always_success_script.clone()).pack())
            .build(),
        Default::default(),
    );
    let input0 = CellInput::new_builder()
        .previous_output(input0_out_point)
        .build();

    let input1_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(10u64.pack())
            .lock(deposit_lock_script.clone())
            .build(),
        Default::default(),
    );
    let input1 = CellInput::new_builder()
        .previous_output(input1_out_point)
        .build();

    let outputs = vec![CellOutput::new_builder()
        .capacity(20u64.pack())
        .lock(always_success_script.clone())
        .build()];
    let outputs_data = vec![Bytes::new()];

    // build transaction
    let tx = TransactionBuilder::default()
        .input(input0)
        .input(input1)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(always_success_script_dep)
        .cell_dep(deposit_lock_dep)
        .build();
    let tx = context.complete_tx(tx);

    let err = context
        .verify_tx(&tx, MAX_CYCLES)
        .unwrap_err();

    assert_error_eq!(err, ScriptError::ValidationFailure(5));
}

#[test]
fn test_collect_1_deposit() {
    let validator_list = Bytes::from(Vec::from_hex("1122334411223344112233441122334411223344112233441122334411223344000000000000000000000000112233445566778899001122334455667788990000000000000000000000000000000000000000000000000000000000000004D2AAAAAAAA").unwrap());
    
    let mut context = Context::default();

    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());
    let always_success_script = context
        .build_script(&always_success_out_point, Default::default())
        .expect("script");
    let always_success_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point)
        .build();

    let bridge_creation_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(10u64.pack())
            .lock(always_success_script.clone())
            .build(),
        Default::default(),
    );
    let bridge_creation_input = CellInput::new_builder()
        .previous_output(bridge_creation_out_point)
        .build();
    
    let tx_hash: &[u8] = &*bridge_creation_input.previous_output().tx_hash().raw_data();
    let index: &[u8] = &*bridge_creation_input.previous_output().index().raw_data();
    let state_id = Bytes::from([tx_hash, index].concat());
    let type_script_args = Bytes::from([&*state_id, &*validator_list].concat());
    
    let bridge_bin: Bytes = Loader::default().load_binary("parent-bridge");
    let bridge_out_point = context.deploy_cell(bridge_bin);
    let bridge_script = context
        .build_script(&bridge_out_point, type_script_args)
        .expect("script");
    let bridge_script_dep = CellDep::new_builder().out_point(bridge_out_point).build();

    let input0_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(10u64.pack())
            .lock(always_success_script.clone())
            .type_(Some(bridge_script.clone()).pack())
            .build(),
        Default::default(),
    );
    let input0 = CellInput::new_builder()
        .previous_output(input0_out_point)
        .build();

    let bridge_script_hash = bridge_script.calc_script_hash().raw_data();
    let deposit_lock_args : Bytes = Bytes::from([&[0; 32], &*bridge_script_hash].concat());
    let deposit_lock_bin: Bytes = Loader::default().load_binary("deposit-lock");
    let deposit_lock_out_point = context.deploy_cell(deposit_lock_bin);
    let deposit_lock_script = context
        .build_script(&deposit_lock_out_point, deposit_lock_args)
        .expect("script");
    let deposit_lock_dep = CellDep::new_builder().out_point(deposit_lock_out_point).build();

    let input1_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(10u64.pack())
            .lock(deposit_lock_script.clone())
            .build(),
        Default::default(),
    );
    let input1 = CellInput::new_builder()
        .previous_output(input1_out_point)
        .build();

    let outputs = vec![CellOutput::new_builder()
        .capacity(20u64.pack())
        .lock(always_success_script.clone())
        .type_(Some(bridge_script.clone()).pack())
        .build()];

    let outputs_data = vec![Bytes::new(); 1];

    let witnesses = vec![Bytes::from(&[1u8][..]), Bytes::new()];

    // build transaction
    let tx = TransactionBuilder::default()
        .input(input0)
        .input(input1)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(always_success_script_dep)
        .cell_dep(deposit_lock_dep)
        .cell_dep(bridge_script_dep)
        .witnesses(witnesses.pack())
        .build();
    let tx = context.complete_tx(tx);

    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}
