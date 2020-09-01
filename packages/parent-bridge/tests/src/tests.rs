use super::*;
use ckb_testtool::{builtin::ALWAYS_SUCCESS, context::Context};
use ckb_tool::ckb_types::{
    bytes::Bytes,
    core::TransactionBuilder,
    packed::*,
    prelude::*,
};
use hex::FromHex;


const MAX_CYCLES: u64 = 10_000_000;

#[test]
fn test_basic() {
    let mut context = Context::default();

    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());
    let lock_script = context
        .build_script(&always_success_out_point, Default::default())
        .expect("script");
    let lock_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point)
        .build();
    
    let contract_bin: Bytes = Loader::default().load_binary("parent-bridge");
    let contract_out_point = context.deploy_cell(contract_bin);
    let bridge_script = context
        .build_script(&contract_out_point, Bytes::from(Vec::from_hex("1122334411223344112233441122334411223344112233441122334411223344000000000000000000000000112233445566778899001122334455667788990000000000000000000000000000000000000000000000000000000000000004D2AAAAAAAA").unwrap()))
        .expect("script");
    let bridge_script_dep = CellDep::new_builder()
        .out_point(contract_out_point)
        .build();
    

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
    
    let outputs = vec![
        CellOutput::new_builder()
            .capacity(0u64.pack())
            .lock(lock_script.clone())
            .type_(Some(bridge_script.clone()).pack())
            .build(),
    ];

    let outputs_data = vec![Bytes::new(); 1];

    let witnesses = vec![Bytes::from(vec![0 as u8; 1]); 1];

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
