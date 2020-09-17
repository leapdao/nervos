use super::*;
use ckb_testtool::{builtin::ALWAYS_SUCCESS, context::Context};
use ckb_tool::ckb_types::{bytes::Bytes, core::TransactionBuilder, packed::*, prelude::*};
use hex::FromHex;

const MAX_CYCLES: u64 = 10_000_000;

// #[test]
// fn test_deploy() {
//     let mut context = Context::default();

//     let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());
//     let lock_script = context
//         .build_script(&always_success_out_point, Default::default())
//         .expect("script");
//     let lock_script_dep = CellDep::new_builder()
//         .out_point(always_success_out_point)
//         .build();

//     let validator_list = Bytes::from(Vec::from_hex("1122334411223344112233441122334411223344112233441122334411223344000000000000000000000000112233445566778899001122334455667788990000000000000000000000000000000000000000000000000000000000000004D2AAAAAAAA").unwrap());

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

//     // in combat the secp256 lock script would check the withness
//     // for a signature, hence we can't use an actionByte here.
//     let witnesses = vec![Bytes::new()];

//     // build transaction
//     let tx = TransactionBuilder::default()
//         .input(input)
//         .outputs(outputs)
//         .outputs_data(outputs_data.pack())
//         .cell_dep(lock_script_dep)
//         .cell_dep(bridge_script_dep)
//         .witnesses(witnesses.pack())
//         .build();
//     let tx = context.complete_tx(tx);

//     // run
//     let cycles = context
//         .verify_tx(&tx, MAX_CYCLES)
//         .expect("pass verification");
//     println!("consume cycles: {}", cycles);
// }

#[test]
fn test_unlock() {
    let mut context = Context::default();
    let validator_list = Bytes::from(Vec::from_hex("f3beac30c498d9e26865f34fcaa57dbb935b0d74").unwrap());

    // mock funding output
    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());
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
    let second_input = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();
    // mock previous bridge output

    let tx_hash: &[u8] = &*second_input.previous_output().tx_hash().raw_data();
    let index: &[u8] = &*second_input.previous_output().index().raw_data();
    let state_id = Bytes::from([tx_hash, index].concat());

    // state id comes in front of validator list
    let type_script_args = Bytes::from([&*state_id, &*validator_list].concat());
    let contract_bin: Bytes = Loader::default().load_binary("parent-bridge");
    let contract_out_point = context.deploy_cell(contract_bin);

    // trustee address to be added to bridge script args

    let bridge_script = context
        .build_script(&contract_out_point, type_script_args)
        .expect("script");
    let bridge_script_dep = CellDep::new_builder().out_point(contract_out_point).build();

    let prev_bridge_output = CellOutput::new_builder()
        .capacity(100u64.pack())
        .lock(lock_script.clone())
        .type_(Some(bridge_script.clone()).pack())
        .build();

    let prev_bridge_outpoint = context.create_cell(
        prev_bridge_output,
        Bytes::new()
    );

    let first_input = CellInput::new_builder()
        .previous_output(prev_bridge_outpoint).build();

    //witness for first input
    let signature = Bytes::from(Vec::from_hex("cba350d5537ab7152a8a6eabd5a499a152b24d72494c4002d8438b4b51da68990e659f9abedf191adeecf4db23cc356d4de9f813432c1bdb43724ecac3ec5bd11c").unwrap());
    let receipt = Bytes::from(Vec::from_hex("0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000143e0824e4a966bb7ce6566e48172dde8c00ecad0000000000000000000000000000000000000000000000001bc16d674ec80000bd7751d7743e0c6e79a2a2b27dceaf80148e92e47206ac643a2ad423f20a8f34").unwrap());
    let action_byte = Bytes::from(Vec::from_hex("00").unwrap());
    let witness = [action_byte, receipt, signature].concat();

    // empty witness for second input
    let witnesses = vec![Bytes::from(witness), Bytes::new(), Bytes::new()];

    let recipient = Bytes::from(Vec::from_hex("f3beac30c498d9e26865f34fcaa57dbb935b0d74").unwrap());
    // TODO: actually use audit delay script
    let audit_delay_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());
    let audit_delay_lock = context
        .build_script(&audit_delay_out_point, recipient) //TODO: add trustee later, once we know it
        .expect("script");

    let outputs = vec![
        CellOutput::new_builder()
            .capacity(90u64.pack())
            .lock(lock_script.clone())
            .type_(Some(bridge_script.clone()).pack())
            .build(),
        CellOutput::new_builder()
            .capacity(10u64.pack())
            .lock(audit_delay_lock.clone())
            .build(),
        CellOutput::new_builder()
            .capacity(5u64.pack())
            .lock(lock_script.clone())
            .build()];

    // TODO: differentiate validator from spent transaction hashes in data from first output
    let outputs_data = vec![Bytes::from(Vec::from_hex("bd7751d7743e0c6e79a2a2b27dceaf80148e92e47206ac643a2ad423f20a8f34").unwrap()), Bytes::new(), Bytes::new()];

    // build transaction
    let tx = TransactionBuilder::default()
        .input(first_input)
        .input(second_input)
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
