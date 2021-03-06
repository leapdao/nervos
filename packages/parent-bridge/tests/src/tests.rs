use super::*;
use ckb_testtool::{context::Context};
use ckb_tool::ckb_types::{bytes::Bytes, core::TransactionBuilder, packed::*, prelude::*};
use ckb_tool::{ckb_error::assert_error_eq, ckb_script::ScriptError};
use hex::FromHex;

const MAX_CYCLES: u64 = 100_000_000;

#[test]
fn test_deploy() {
    let mut context = Context::default();

    let always_success_bin: Bytes = Loader::default().load_binary("anyone-can-spend");
    let always_success_out_point = context.deploy_cell(always_success_bin);
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
    let trustee = Bytes::from(Vec::from_hex("9999999999999999999999999999999999999999999999999999999999999999").unwrap());
    let type_script_args = Bytes::from([&*state_id, &*trustee, &*validator_list].concat());

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

//     let trustee = Bytes::from(Vec::from_hex("9999999999999999999999999999999999999999999999999999999999999999").unwrap());
    // let type_script_args = Bytes::from([&*state_id, &*trustee, &*validator_list].concat());

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

//     let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();

//     assert_error_eq!(err, ScriptError::ValidationFailure(5).input_lock_script(0));
// }


#[test]
fn test_wrong_validator_list_length() {
    let mut context = Context::default();
    let validator_list = Bytes::from(Vec::from_hex("112233441122411223344112233441122334411223344112233441122334400000000000000000000000011223344556677889900112233445566778899000000000000000000000000000000000000000000000000000000000000004D2AAAAAAAA").unwrap());
    // mock funding output
    let always_success_bin: Bytes = Loader::default().load_binary("anyone-can-spend");
    let always_success_out_point = context.deploy_cell(always_success_bin);
    let lock_script = context
        .build_script(&always_success_out_point, Default::default())
        .expect("script");
    let lock_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point)
        .build();

    // create input from funding output
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

    let trustee = Bytes::from(Vec::from_hex("9999999999999999999999999999999999999999999999999999999999999999").unwrap());
    let type_script_args = Bytes::from([&*state_id, &*trustee, &*validator_list].concat());

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

    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();

    assert_error_eq!(err, ScriptError::ValidationFailure(6).output_type_script(0));
}

#[test]
fn test_wrong_lock_script() {
    let mut context = Context::default();

    let always_success_bin: Bytes = Loader::default().load_binary("anyone-can-spend");
    let always_success_out_point = context.deploy_cell(always_success_bin);
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

    let trustee = Bytes::from(Vec::from_hex("9999999999999999999999999999999999999999999999999999999999999999").unwrap());
    let type_script_args = Bytes::from([&*state_id, &*trustee, &*validator_list].concat());

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

    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();

    assert_error_eq!(err, ScriptError::ValidationFailure(7).output_type_script(0));
}

#[test]
fn test_wrong_type_script() {
    let mut context = Context::default();

    let always_success_bin: Bytes = Loader::default().load_binary("anyone-can-spend");
    let always_success_out_point = context.deploy_cell(always_success_bin);
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

    let trustee = Bytes::from(Vec::from_hex("9999999999999999999999999999999999999999999999999999999999999999").unwrap());
    let type_script_args = Bytes::from([&*state_id, &*trustee, &*validator_list].concat());

    let contract_bin: Bytes = Loader::default().load_binary("parent-bridge");
    let contract_out_point = context.deploy_cell(contract_bin);
    let bridge_script = context
        .build_script(&contract_out_point, type_script_args)
        .expect("script");
    let bridge_script_dep = CellDep::new_builder().out_point(contract_out_point).build();

    let outputs = vec![
        CellOutput::new_builder()
            .capacity(0u64.pack())
            .lock(lock_script.clone())
            .type_(Some(lock_script.clone()).pack())
            .build(),
        CellOutput::new_builder()
            .capacity(0u64.pack())
            .lock(lock_script.clone())
            .type_(Some(bridge_script.clone()).pack())
            .build(),
    ];

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
    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();

    assert_error_eq!(err, ScriptError::ValidationFailure(8).output_type_script(1));
}

#[test]
fn test_data_length_not_zero() {
    let mut context = Context::default();

    let always_success_bin: Bytes = Loader::default().load_binary("anyone-can-spend");
    let always_success_out_point = context.deploy_cell(always_success_bin);
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

    let trustee = Bytes::from(Vec::from_hex("9999999999999999999999999999999999999999999999999999999999999999").unwrap());
    let type_script_args = Bytes::from([&*state_id, &*trustee, &*validator_list].concat());

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

    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();

    assert_error_eq!(err, ScriptError::ValidationFailure(9).output_type_script(0));
}

#[test]
fn test_wrong_state_id() {
    let mut context = Context::default();

    let always_success_bin: Bytes = Loader::default().load_binary("anyone-can-spend");
    let always_success_out_point = context.deploy_cell(always_success_bin);
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

    let trustee = Bytes::from(Vec::from_hex("9999999999999999999999999999999999999999999999999999999999999999").unwrap());
    let type_script_args = Bytes::from([&*state_id, &*trustee, &*validator_list].concat());

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

    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();

    assert_error_eq!(err, ScriptError::ValidationFailure(10).output_type_script(0));
}

#[test]
fn test_too_many_type_outputs() {
    let mut context = Context::default();

    let always_success_bin: Bytes = Loader::default().load_binary("anyone-can-spend");
    let always_success_out_point = context.deploy_cell(always_success_bin);
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

    let trustee = Bytes::from(Vec::from_hex("9999999999999999999999999999999999999999999999999999999999999999").unwrap());
    let type_script_args = Bytes::from([&*state_id, &*trustee, &*validator_list].concat());

    let contract_bin: Bytes = Loader::default().load_binary("parent-bridge");
    let contract_out_point = context.deploy_cell(contract_bin);
    let bridge_script = context
        .build_script(&contract_out_point, type_script_args)
        .expect("script");
    let bridge_script_dep = CellDep::new_builder().out_point(contract_out_point).build();

    let outputs = vec![
        CellOutput::new_builder()
            .capacity(0u64.pack())
            .lock(lock_script.clone())
            .type_(Some(bridge_script.clone()).pack())
            .build(),
        CellOutput::new_builder()
            .capacity(0u64.pack())
            .lock(lock_script.clone())
            .type_(Some(bridge_script.clone()).pack())
            .build(),
    ];

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

    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();

    assert_error_eq!(err, ScriptError::ValidationFailure(11).output_type_script(0));
}

#[test]
fn test_deposit_with_lock() {
    let mut context = Context::default();

    let always_success_bin: Bytes = Loader::default().load_binary("anyone-can-spend");
    let always_success_out_point = context.deploy_cell(always_success_bin);
    let always_success_script = context
        .build_script(&always_success_out_point, Default::default())
        .expect("script");
    let always_success_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point)
        .build();

    let always_success_script_hash = always_success_script.calc_script_hash().raw_data();

    let deposit_lock_args: Bytes = Bytes::from([&*always_success_script_hash, &[0; 32]].concat());
    let deposit_lock_bin: Bytes = Loader::default().load_binary("deposit-lock");
    let deposit_lock_out_point = context.deploy_cell(deposit_lock_bin);
    let deposit_lock_script = context
        .build_script(&deposit_lock_out_point, deposit_lock_args)
        .expect("script");
    let deposit_lock_dep = CellDep::new_builder()
        .out_point(deposit_lock_out_point)
        .build();

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

    let always_success_bin: Bytes = Loader::default().load_binary("anyone-can-spend");
    let always_success_out_point = context.deploy_cell(always_success_bin);
    let always_success_script = context
        .build_script(&always_success_out_point, Default::default())
        .expect("script");
    let always_success_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point)
        .build();

    let always_success_script_hash = always_success_script.calc_script_hash().raw_data();

    let deposit_lock_args: Bytes = Bytes::from([&[0; 32], &*always_success_script_hash].concat());
    let deposit_lock_bin: Bytes = Loader::default().load_binary("deposit-lock");
    let deposit_lock_out_point = context.deploy_cell(deposit_lock_bin);
    let deposit_lock_script = context
        .build_script(&deposit_lock_out_point, deposit_lock_args)
        .expect("script");
    let deposit_lock_dep = CellDep::new_builder()
        .out_point(deposit_lock_out_point)
        .build();

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

    let always_success_bin: Bytes = Loader::default().load_binary("anyone-can-spend");
    let always_success_out_point = context.deploy_cell(always_success_bin);
    let always_success_script = context
        .build_script(&always_success_out_point, Default::default())
        .expect("script");
    let always_success_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point)
        .build();

    let deposit_lock_args: Bytes = Bytes::from([[0; 32], [0; 32]].concat());
    let deposit_lock_bin: Bytes = Loader::default().load_binary("deposit-lock");
    let deposit_lock_out_point = context.deploy_cell(deposit_lock_bin);
    let deposit_lock_script = context
        .build_script(&deposit_lock_out_point, deposit_lock_args)
        .expect("script");
    let deposit_lock_dep = CellDep::new_builder()
        .out_point(deposit_lock_out_point)
        .build();

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

    let err = context.verify_tx(&tx, MAX_CYCLES).unwrap_err();

    assert_error_eq!(err, ScriptError::ValidationFailure(5).input_lock_script(1));
}

#[test]
fn test_collect_1_deposit() {
    let validator_list = Bytes::from(Vec::from_hex("f3beac30c498d9e26865f34fcaa57dbb935b0d74").unwrap());

    let mut context = Context::default();

    let always_success_bin: Bytes = Loader::default().load_binary("anyone-can-spend");
    let always_success_out_point = context.deploy_cell(always_success_bin);
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
    let trustee = Bytes::from(Vec::from_hex("9999999999999999999999999999999999999999999999999999999999999999").unwrap());
    let type_script_args = Bytes::from([&*state_id, &*trustee, &*validator_list].concat());

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
    let deposit_lock_args: Bytes = Bytes::from([&[0; 32], &*bridge_script_hash].concat());
    let deposit_lock_bin: Bytes = Loader::default().load_binary("deposit-lock");
    let deposit_lock_out_point = context.deploy_cell(deposit_lock_bin);
    let deposit_lock_script = context
        .build_script(&deposit_lock_out_point, deposit_lock_args)
        .expect("script");
    let deposit_lock_dep = CellDep::new_builder()
        .out_point(deposit_lock_out_point)
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

#[test]
fn test_collect_3_deposits() {
    let validator_list = Bytes::from(Vec::from_hex("f3beac30c498d9e26865f34fcaa57dbb935b0d74").unwrap());

    let mut context = Context::default();

    let always_success_bin: Bytes = Loader::default().load_binary("anyone-can-spend");
    let always_success_out_point = context.deploy_cell(always_success_bin);
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
    let trustee = Bytes::from(Vec::from_hex("9999999999999999999999999999999999999999999999999999999999999999").unwrap());
    let type_script_args = Bytes::from([&*state_id, &*trustee, &*validator_list].concat());

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
    let deposit_lock_args: Bytes = Bytes::from([&[0; 32], &*bridge_script_hash].concat());
    let deposit_lock_bin: Bytes = Loader::default().load_binary("deposit-lock");
    let deposit_lock_out_point = context.deploy_cell(deposit_lock_bin);
    let deposit_lock_script = context
        .build_script(&deposit_lock_out_point, deposit_lock_args)
        .expect("script");
    let deposit_lock_dep = CellDep::new_builder()
        .out_point(deposit_lock_out_point)
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

    let input2_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(20u64.pack())
            .lock(deposit_lock_script.clone())
            .build(),
        Default::default(),
    );
    let input2 = CellInput::new_builder()
        .previous_output(input2_out_point)
        .build();

    let input3_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(30u64.pack())
            .lock(deposit_lock_script.clone())
            .build(),
        Default::default(),
    );
    let input3 = CellInput::new_builder()
        .previous_output(input3_out_point)
        .build();

    let outputs = vec![CellOutput::new_builder()
        .capacity(70u64.pack())
        .lock(always_success_script.clone())
        .type_(Some(bridge_script.clone()).pack())
        .build()];

    let outputs_data = vec![Bytes::new(); 1];

    let witnesses = vec![Bytes::from(&[1u8][..]), Bytes::new()];

    // build transaction
    let tx = TransactionBuilder::default()
        .input(input0)
        .input(input1)
        .input(input2)
        .input(input3)
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

#[test]
fn test_collect_desposits_invalid_sum() {
    let validator_list = Bytes::from(Vec::from_hex("f3beac30c498d9e26865f34fcaa57dbb935b0d74").unwrap());

    let mut context = Context::default();

    let always_success_bin: Bytes = Loader::default().load_binary("anyone-can-spend");
    let always_success_out_point = context.deploy_cell(always_success_bin);
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
    let trustee = Bytes::from(Vec::from_hex("9999999999999999999999999999999999999999999999999999999999999999").unwrap());
    let type_script_args = Bytes::from([&*state_id, &*trustee, &*validator_list].concat());

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
    let deposit_lock_args: Bytes = Bytes::from([&[0; 32], &*bridge_script_hash].concat());
    let deposit_lock_bin: Bytes = Loader::default().load_binary("deposit-lock");
    let deposit_lock_out_point = context.deploy_cell(deposit_lock_bin);
    let deposit_lock_script = context
        .build_script(&deposit_lock_out_point, deposit_lock_args)
        .expect("script");
    let deposit_lock_dep = CellDep::new_builder()
        .out_point(deposit_lock_out_point)
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

    let input2_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(20u64.pack())
            .lock(deposit_lock_script.clone())
            .build(),
        Default::default(),
    );
    let input2 = CellInput::new_builder()
        .previous_output(input2_out_point)
        .build();

    let input3_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(30u64.pack())
            .lock(deposit_lock_script.clone())
            .build(),
        Default::default(),
    );
    let input3 = CellInput::new_builder()
        .previous_output(input3_out_point)
        .build();

    let outputs = vec![CellOutput::new_builder()
        .capacity(80u64.pack())
        .lock(always_success_script.clone())
        .type_(Some(bridge_script.clone()).pack())
        .build()];

    let outputs_data = vec![Bytes::new(); 1];

    let witnesses = vec![Bytes::from(&[1u8][..]), Bytes::new()];

    // build transaction
    let tx = TransactionBuilder::default()
        .input(input0)
        .input(input1)
        .input(input2)
        .input(input3)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(always_success_script_dep)
        .cell_dep(deposit_lock_dep)
        .cell_dep(bridge_script_dep)
        .witnesses(witnesses.pack())
        .build();
    let tx = context.complete_tx(tx);

    // run
    let err = context
        .verify_tx(&tx, MAX_CYCLES)
        .unwrap_err();

    assert_error_eq!(err, ScriptError::ValidationFailure(16).input_type_script(0));
}

#[test]
fn test_collect_deposit_fiddling_with_data() {
    let validator_list = Bytes::from(Vec::from_hex("f3beac30c498d9e26865f34fcaa57dbb935b0d74").unwrap());

    let mut context = Context::default();

    let always_success_bin: Bytes = Loader::default().load_binary("anyone-can-spend");
    let always_success_out_point = context.deploy_cell(always_success_bin);
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
    let trustee = Bytes::from(Vec::from_hex("9999999999999999999999999999999999999999999999999999999999999999").unwrap());
    let type_script_args = Bytes::from([&*state_id, &*trustee, &*validator_list].concat());

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
    let deposit_lock_args: Bytes = Bytes::from([&[0; 32], &*bridge_script_hash].concat());
    let deposit_lock_bin: Bytes = Loader::default().load_binary("deposit-lock");
    let deposit_lock_out_point = context.deploy_cell(deposit_lock_bin);
    let deposit_lock_script = context
        .build_script(&deposit_lock_out_point, deposit_lock_args)
        .expect("script");
    let deposit_lock_dep = CellDep::new_builder()
        .out_point(deposit_lock_out_point)
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
        .type_(Some(bridge_script.clone()).pack())
        .build()];

    let outputs_data = vec![Bytes::from(&[1u8][..])];

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
    let err = context
        .verify_tx(&tx, MAX_CYCLES)
        .unwrap_err();

    assert_error_eq!(err, ScriptError::ValidationFailure(17).input_type_script(0));
}

#[test]
fn test_dissolve_bridge_success() {
    let mut context = Context::default();

    let always_success_bin: Bytes = Loader::default().load_binary("anyone-can-spend");
    let always_success_out_point = context.deploy_cell(always_success_bin);
    let lock_script = context
        .build_script(&always_success_out_point, Default::default())
        .expect("script");
    let lock_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point)
        .build();

    let validator_list = Bytes::from(Vec::from_hex("1122334411223343241123344112233441122344112233441122334411223344000000000000000000000000112233445566778899001122334455667788990000000000000000000000000000000000000000000000000000000000000004D2AAAAAAAA").unwrap());
    
    let bridge_creation_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(10u64.pack())
            .lock(lock_script.clone())
            .build(),
        Default::default(),
    );
    let bridge_creation_input = CellInput::new_builder()
        .previous_output(bridge_creation_out_point)
        .build();

    let tx_hash: &[u8] = &*bridge_creation_input.previous_output().tx_hash().raw_data();
    let index: &[u8] = &*bridge_creation_input.previous_output().index().raw_data();
    let state_id = Bytes::from([tx_hash, index].concat());
    let trustee = lock_script.calc_script_hash().raw_data();
    let type_script_args = Bytes::from([&*state_id, &*trustee, &*validator_list].concat());

    let bridge_bin: Bytes = Loader::default().load_binary("parent-bridge");
    let bridge_out_point = context.deploy_cell(bridge_bin);
    let bridge_script = context
        .build_script(&bridge_out_point, type_script_args)
        .expect("script");
    let bridge_script_dep = CellDep::new_builder().out_point(bridge_out_point).build();

    let input0_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(10u64.pack())
            .lock(lock_script.clone())
            .type_(Some(bridge_script.clone()).pack())
            .build(),
        Default::default(),
    );

    let input0 = CellInput::new_builder()
        .previous_output(input0_out_point)
        .build();
    
    let input_out_point1 = context.create_cell(
        CellOutput::new_builder()
            .capacity(10u64.pack())
            .lock(lock_script.clone())
            .build(),
        Default::default(),
    );

    let input1 = CellInput::new_builder()
        .previous_output(input_out_point1)
        .build();

    let outputs = vec![
        CellOutput::new_builder()
            .capacity(5u64.pack())
            .lock(lock_script.clone())
            .build(),
        CellOutput::new_builder()
            .capacity(5u64.pack())
            .lock(lock_script.clone())
            .build()
    ];

    let outputs_data = vec![Bytes::new(); 2];

    // in combat the secp256 lock script would check the withness
    // for a signature, hence we can't use an actionByte here.
    let witnesses = vec![Bytes::from(&[2u8][..]), Bytes::new()];

    // build transaction
    let tx = TransactionBuilder::default()
        .input(input0)
        .input(input1)
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
}

#[test]
fn test_not_signed_by_trustee() {
    let mut context = Context::default();

    let always_success_bin: Bytes = Loader::default().load_binary("anyone-can-spend");
    let always_success_out_point = context.deploy_cell(always_success_bin);
    let lock_script = context
        .build_script(&always_success_out_point, Default::default())
        .expect("script");
    let lock_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point)
        .build();

    let validator_list = Bytes::from(Vec::from_hex("1122334411223343241123344112233441122344112233441122334411223344000000000000000000000000112233445566778899001122334455667788990000000000000000000000000000000000000000000000000000000000000004D2AAAAAAAA").unwrap());
    
    let bridge_creation_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(10u64.pack())
            .lock(lock_script.clone())
            .build(),
        Default::default(),
    );
    let bridge_creation_input = CellInput::new_builder()
        .previous_output(bridge_creation_out_point)
        .build();

    let tx_hash: &[u8] = &*bridge_creation_input.previous_output().tx_hash().raw_data();
    let index: &[u8] = &*bridge_creation_input.previous_output().index().raw_data();
    let state_id = Bytes::from([tx_hash, index].concat());
    let trustee = Bytes::from(Vec::from_hex("9999999999999999999999999999999999999999999999999999999999999999").unwrap());
    let type_script_args = Bytes::from([&*state_id, &*trustee, &*validator_list].concat());

    let bridge_bin: Bytes = Loader::default().load_binary("parent-bridge");
    let bridge_out_point = context.deploy_cell(bridge_bin);
    let bridge_script = context
        .build_script(&bridge_out_point, type_script_args)
        .expect("script");
    let bridge_script_dep = CellDep::new_builder().out_point(bridge_out_point).build();

    let input0_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(10u64.pack())
            .lock(lock_script.clone())
            .type_(Some(bridge_script.clone()).pack())
            .build(),
        Default::default(),
    );

    let input0 = CellInput::new_builder()
        .previous_output(input0_out_point)
        .build();
    
    let input_out_point1 = context.create_cell(
        CellOutput::new_builder()
            .capacity(10u64.pack())
            .lock(lock_script.clone())
            .build(),
        Default::default(),
    );

    let input1 = CellInput::new_builder()
        .previous_output(input_out_point1)
        .build();

    let outputs = vec![
        CellOutput::new_builder()
            .capacity(5u64.pack())
            .lock(lock_script.clone())
            .build(),
        CellOutput::new_builder()
            .capacity(5u64.pack())
            .lock(lock_script.clone())
            .build()
    ];

    let outputs_data = vec![Bytes::new(); 2];

    // in combat the secp256 lock script would check the withness
    // for a signature, hence we can't use an actionByte here.
    let witnesses = vec![Bytes::from(&[2u8][..]), Bytes::new()];

    // build transaction
    let tx = TransactionBuilder::default()
        .input(input0)
        .input(input1)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(lock_script_dep)
        .cell_dep(bridge_script_dep)
        .witnesses(witnesses.pack())
        .build();
    let tx = context.complete_tx(tx);

    // run
    let err = context
        .verify_tx(&tx, MAX_CYCLES)
        .unwrap_err();

    assert_error_eq!(err, ScriptError::ValidationFailure(18).input_type_script(0));
}

#[test]
fn test_bridge_was_not_dissolved() {
    let mut context = Context::default();

    let always_success_bin: Bytes = Loader::default().load_binary("anyone-can-spend");
    let always_success_out_point = context.deploy_cell(always_success_bin);
    let lock_script = context
        .build_script(&always_success_out_point, Default::default())
        .expect("script");
    let lock_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point)
        .build();

    let validator_list = Bytes::from(Vec::from_hex("1122334411223343241123344112233441122344112233441122334411223344000000000000000000000000112233445566778899001122334455667788990000000000000000000000000000000000000000000000000000000000000004D2AAAAAAAA").unwrap());
    
    let bridge_creation_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(10u64.pack())
            .lock(lock_script.clone())
            .build(),
        Default::default(),
    );
    let bridge_creation_input = CellInput::new_builder()
        .previous_output(bridge_creation_out_point)
        .build();

    let tx_hash: &[u8] = &*bridge_creation_input.previous_output().tx_hash().raw_data();
    let index: &[u8] = &*bridge_creation_input.previous_output().index().raw_data();
    let state_id = Bytes::from([tx_hash, index].concat());
    let trustee = lock_script.calc_script_hash().raw_data();
    let type_script_args = Bytes::from([&*state_id, &*trustee, &*validator_list].concat());

    let bridge_bin: Bytes = Loader::default().load_binary("parent-bridge");
    let bridge_out_point = context.deploy_cell(bridge_bin);
    let bridge_script = context
        .build_script(&bridge_out_point, type_script_args)
        .expect("script");
    let bridge_script_dep = CellDep::new_builder().out_point(bridge_out_point).build();

    let input0_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(10u64.pack())
            .lock(lock_script.clone())
            .type_(Some(bridge_script.clone()).pack())
            .build(),
        Default::default(),
    );

    let input0 = CellInput::new_builder()
        .previous_output(input0_out_point)
        .build();
    
    let input_out_point1 = context.create_cell(
        CellOutput::new_builder()
            .capacity(10u64.pack())
            .lock(lock_script.clone())
            .build(),
        Default::default(),
    );

    let input1 = CellInput::new_builder()
        .previous_output(input_out_point1)
        .build();

    let outputs = vec![CellOutput::new_builder()
        .capacity(10u64.pack())
        .lock(lock_script.clone())
        .type_(Some(bridge_script.clone()).pack())
        .build()];

    let outputs_data = vec![Bytes::new(); 1];

    // in combat the secp256 lock script would check the withness
    // for a signature, hence we can't use an actionByte here.
    let witnesses = vec![Bytes::from(&[2u8][..]), Bytes::new()];

    // build transaction
    let tx = TransactionBuilder::default()
        .input(input0)
        .input(input1)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(lock_script_dep)
        .cell_dep(bridge_script_dep)
        .witnesses(witnesses.pack())
        .build();
    let tx = context.complete_tx(tx);

    // run
    let err = context
        .verify_tx(&tx, MAX_CYCLES)
        .unwrap_err();

    assert_error_eq!(err, ScriptError::ValidationFailure(19).input_type_script(0));
}

#[test]
fn test_leftover_capacity() {
    let mut context = Context::default();

    let always_success_bin: Bytes = Loader::default().load_binary("anyone-can-spend");
    let always_success_out_point = context.deploy_cell(always_success_bin);
    let lock_script = context
        .build_script(&always_success_out_point, Default::default())
        .expect("script");
    let lock_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point)
        .build();

    let validator_list = Bytes::from(Vec::from_hex("1122334411223343241123344112233441122344112233441122334411223344000000000000000000000000112233445566778899001122334455667788990000000000000000000000000000000000000000000000000000000000000004D2AAAAAAAA").unwrap());
    
    let bridge_creation_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(10u64.pack())
            .lock(lock_script.clone())
            .build(),
        Default::default(),
    );
    let bridge_creation_input = CellInput::new_builder()
        .previous_output(bridge_creation_out_point)
        .build();

    let tx_hash: &[u8] = &*bridge_creation_input.previous_output().tx_hash().raw_data();
    let index: &[u8] = &*bridge_creation_input.previous_output().index().raw_data();
    let state_id = Bytes::from([tx_hash, index].concat());
    let trustee = lock_script.calc_script_hash().raw_data();
    let type_script_args = Bytes::from([&*state_id, &*trustee, &*validator_list].concat());

    let bridge_bin: Bytes = Loader::default().load_binary("parent-bridge");
    let bridge_out_point = context.deploy_cell(bridge_bin);
    let bridge_script = context
        .build_script(&bridge_out_point, type_script_args)
        .expect("script");
    let bridge_script_dep = CellDep::new_builder().out_point(bridge_out_point).build();

    let input0_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(10u64.pack())
            .lock(lock_script.clone())
            .type_(Some(bridge_script.clone()).pack())
            .build(),
        Default::default(),
    );

    let input0 = CellInput::new_builder()
        .previous_output(input0_out_point)
        .build();
    
    let input_out_point1 = context.create_cell(
        CellOutput::new_builder()
            .capacity(10u64.pack())
            .lock(lock_script.clone())
            .build(),
        Default::default(),
    );

    let input1 = CellInput::new_builder()
        .previous_output(input_out_point1)
        .build();

    let outputs = vec![
        CellOutput::new_builder()
            .capacity(5u64.pack())
            .lock(lock_script.clone())
            .build(),
        CellOutput::new_builder()
            .capacity(1u64.pack())
            .lock(lock_script.clone())
            .build()
    ];

    let outputs_data = vec![Bytes::new(); 2];

    // in combat the secp256 lock script would check the withness
    // for a signature, hence we can't use an actionByte here.
    let witnesses = vec![Bytes::from(&[2u8][..]), Bytes::new()];

    // build transaction
    let tx = TransactionBuilder::default()
        .input(input0)
        .input(input1)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(lock_script_dep)
        .cell_dep(bridge_script_dep)
        .witnesses(witnesses.pack())
        .build();
    let tx = context.complete_tx(tx);

    // run
    let err = context
        .verify_tx(&tx, MAX_CYCLES)
        .unwrap_err();

    assert_error_eq!(err, ScriptError::ValidationFailure(20).input_type_script(0));
}

#[test]
fn test_not_enough_time_passed() {
    let mut context = Context::default();

    let always_success_bin: Bytes = Loader::default().load_binary("anyone-can-spend");
    let always_success_out_point = context.deploy_cell(always_success_bin);
    let lock_script = context
        .build_script(&always_success_out_point, Default::default())
        .expect("script");
    let lock_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point)
        .build();

    let contract_bin: Bytes = Loader::default().load_binary("audit-delay");
    let contract_out_point = context.deploy_cell(contract_bin);

    let lock_script_hash = lock_script.calc_script_hash().raw_data();
    let audit_delay_lock_args: Bytes = Bytes::from([&*lock_script_hash, &[0; 32], &100u64.to_be_bytes()].concat());

    let audit_delay_script = context
        .build_script(&contract_out_point, audit_delay_lock_args)
        .expect("script");
    let audit_delay_script_dep = CellDep::new_builder().out_point(contract_out_point).build();
    
    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(10u64.pack())
            .lock(audit_delay_script.clone())
            .build(),
        Default::default(),
    );
    let input = CellInput::new_builder()
        .previous_output(input_out_point.clone())
        .build();
    
    let outputs = vec![CellOutput::new_builder()
        .capacity(0u64.pack())
        .lock(lock_script.clone())
        .build()];

    let outputs_data = vec![Bytes::new(); 1];

    // in combat the secp256 lock script would check the witness
    // for a signature, hence we can't use an actionByte here.
    let witnesses = vec![Bytes::new()];

    let h1 = Header::new_builder()
        .raw(RawHeader::new_builder().number(1u64.pack()).timestamp(500u64.pack()).build())
        .build()
        .into_view();
    let h2 = Header::new_builder()
        .raw(RawHeader::new_builder().number(2u64.pack()).timestamp(550u64.pack()).build())
        .build()
        .into_view();

    context.insert_header(h1.clone());
    context.insert_header(h2.clone());
    context.link_cell_with_block(input_out_point, h1.hash(), 0);

    // build transaction
    let tx = TransactionBuilder::default()
        .input(input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(lock_script_dep)
        .cell_dep(audit_delay_script_dep)
        .header_dep(h1.hash())
        .header_dep(h2.hash())
        .witnesses(witnesses.pack())
        .build();
    let tx = context.complete_tx(tx);

    let err = context
        .verify_tx(&tx, MAX_CYCLES)
        .unwrap_err();

    assert_error_eq!(err, ScriptError::ValidationFailure(5).input_lock_script(0));
}

#[test]
fn test_not_spent_with_owner_input() {
    let mut context = Context::default();

    let always_success_bin: Bytes = Loader::default().load_binary("anyone-can-spend");
    let always_success_out_point = context.deploy_cell(always_success_bin);
    let lock_script = context
        .build_script(&always_success_out_point, Default::default())
        .expect("script");
    let lock_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point)
        .build();

    let contract_bin: Bytes = Loader::default().load_binary("audit-delay");
    let contract_out_point = context.deploy_cell(contract_bin);

    let audit_delay_lock_args: Bytes = Bytes::from([&[0; 32][..], &[0; 32][..], &100u64.to_be_bytes()[..]].concat());

    let audit_delay_script = context
        .build_script(&contract_out_point, audit_delay_lock_args)
        .expect("script");
    let audit_delay_script_dep = CellDep::new_builder().out_point(contract_out_point).build();
    
    let input0_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(10u64.pack())
            .lock(lock_script.clone())
            .build(),
        Default::default(),
    );
    let input0 = CellInput::new_builder()
            .previous_output(input0_out_point)
            .build();

    let input1_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(10u64.pack())
            .lock(audit_delay_script.clone())
            .build(),
        Default::default(),
    );
    let input1 = CellInput::new_builder()
        .previous_output(input1_out_point.clone())
        .build();
    
    let outputs = vec![CellOutput::new_builder()
        .capacity(0u64.pack())
        .lock(lock_script.clone())
        .build()];

    let outputs_data = vec![Bytes::new(); 1];

    // in combat the secp256 lock script would check the witness
    // for a signature, hence we can't use an actionByte here.
    let witnesses = vec![Bytes::new()];

    let h1 = Header::new_builder()
        .raw(RawHeader::new_builder().number(1u64.pack()).timestamp(500u64.pack()).build())
        .build()
        .into_view();
    let h2 = Header::new_builder()
        .raw(RawHeader::new_builder().number(2u64.pack()).timestamp(610u64.pack()).build())
        .build()
        .into_view();

    context.insert_header(h1.clone());
    context.insert_header(h2.clone());
    context.link_cell_with_block(input1_out_point, h1.hash(), 0);

    // build transaction
    let tx = TransactionBuilder::default()
        .input(input0)
        .input(input1)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(lock_script_dep)
        .cell_dep(audit_delay_script_dep)
        .header_dep(h1.hash())
        .header_dep(h2.hash())
        .witnesses(witnesses.pack())
        .build();
    let tx = context.complete_tx(tx);

    let err = context
        .verify_tx(&tx, MAX_CYCLES)
        .unwrap_err();

    assert_error_eq!(err, ScriptError::ValidationFailure(6).input_lock_script(1));
}

#[test]
fn test_wrong_script_args_length() {
    let mut context = Context::default();

    let always_success_bin: Bytes = Loader::default().load_binary("anyone-can-spend");
    let always_success_out_point = context.deploy_cell(always_success_bin);
    let lock_script = context
        .build_script(&always_success_out_point, Default::default())
        .expect("script");
    let lock_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point)
        .build();

    let contract_bin: Bytes = Loader::default().load_binary("audit-delay");
    let contract_out_point = context.deploy_cell(contract_bin);

    let lock_script_hash = lock_script.calc_script_hash().raw_data();
    let audit_delay_lock_args: Bytes = Bytes::from([&*lock_script_hash, &[0; 64], &100u64.to_be_bytes()].concat());

    let audit_delay_script = context
        .build_script(&contract_out_point, audit_delay_lock_args)
        .expect("script");
    let audit_delay_script_dep = CellDep::new_builder().out_point(contract_out_point).build();
    
    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(10u64.pack())
            .lock(audit_delay_script.clone())
            .build(),
        Default::default(),
    );
    let input = CellInput::new_builder()
        .previous_output(input_out_point.clone())
        .build();
    
    let outputs = vec![CellOutput::new_builder()
        .capacity(0u64.pack())
        .lock(lock_script.clone())
        .build()];

    let outputs_data = vec![Bytes::new(); 1];

    // in combat the secp256 lock script would check the witness
    // for a signature, hence we can't use an actionByte here.
    let witnesses = vec![Bytes::new()];

    let h1 = Header::new_builder()
        .raw(RawHeader::new_builder().number(1u64.pack()).timestamp(500u64.pack()).build())
        .build()
        .into_view();
    let h2 = Header::new_builder()
        .raw(RawHeader::new_builder().number(2u64.pack()).timestamp(610u64.pack()).build())
        .build()
        .into_view();

    context.insert_header(h1.clone());
    context.insert_header(h2.clone());
    context.link_cell_with_block(input_out_point, h1.hash(), 0);

    // build transaction
    let tx = TransactionBuilder::default()
        .input(input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(lock_script_dep)
        .cell_dep(audit_delay_script_dep)
        .header_dep(h1.hash())
        .header_dep(h2.hash())
        .witnesses(witnesses.pack())
        .build();
    let tx = context.complete_tx(tx);

    let err = context
        .verify_tx(&tx, MAX_CYCLES)
        .unwrap_err();

    assert_error_eq!(err, ScriptError::ValidationFailure(7).input_lock_script(0));
}

