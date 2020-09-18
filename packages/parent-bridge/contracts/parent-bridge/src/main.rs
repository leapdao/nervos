#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]

// Import from `core` instead of from `std` since we are in no-std mode
use core::result::Result;

// Import heap related library from `alloc`
// https://doc.rust-lang.org/alloc/index.html
use alloc::vec::Vec;

use ckb_std::{
    ckb_constants::Source,
    ckb_types::bytes::Bytes,
    debug, default_alloc, entry,
    error::SysError,
    high_level::{
        load_cell_data, load_cell_lock, load_cell_type, load_cell_type_hash, load_input_out_point,
        load_script, load_script_hash, QueryIter, load_transaction, load_cell_capacity
    },
};

entry!(entry);
default_alloc!();

/// Program entry
fn entry() -> i8 {
    // Call main function and return error code
    match main() {
        Ok(_) => 0,
        Err(err) => err as i8,
    }
}

// FROM rfc
// const ANYONE_CAN_PAY_CODE_HASH : [u8; 32] = [
//     0x86, 0xa1, 0xc6, 0x98, 0x7a, 0x4a, 0xcb, 0xe1, 0xa8, 0x87, 0xcc, 0xa4, 0xc9, 0xdd,
//     0x2a, 0xc9, 0xfc, 0xb0, 0x74, 0x05, 0xbb, 0xed, 0xa5, 0x1b, 0x86, 0x1b, 0x18, 0xbb,
//     0xf7, 0x49, 0x2c, 0x4b
// ];

// FROM tests
const ANYONE_CAN_PAY_CODE_HASH: [u8; 32] = [
    230, 131, 176, 65, 57, 52, 71, 104, 52, 132, 153, 194, 62, 177, 50, 109, 90, 82, 214, 219, 0,
    108, 13, 47, 236, 224, 10, 131, 31, 54, 96, 215,
];

const ADDRESS_LEN: usize = 20;

/// Error
#[repr(i8)]
enum Error {
    IndexOutOfBound = 1,
    ItemMissing = 2,
    LengthNotEnough = 3,
    Encoding = 4,
    StateTransitionDoesNotExist = 5,
    InvalidArgsEncoding = 6,
    WrongLockScript = 7,
    WrongTypeScript = 8,
    DataLengthNotZero = 9,
    WrongStateId = 10,
    TooManyTypeOutputs = 11,
    EmptyValidatorList = 12,
    InvalidPayoutAmount = 13,
    // Add customized errors here...
}

impl From<SysError> for Error {
    fn from(err: SysError) -> Self {
        use SysError::*;
        match err {
            IndexOutOfBound => Self::IndexOutOfBound,
            ItemMissing => Self::ItemMissing,
            LengthNotEnough(_) => Self::LengthNotEnough,
            Encoding => Self::Encoding,
            Unknown(err_code) => panic!("unexpected sys error {}", err_code),
        }
    }
}

type Address = [u8; ADDRESS_LEN];
type Receipt = [u8; 128];
type Signature = [u8; 65];

enum StateTransition {
    DeployBridge { validators: Vec<Address>, id: Bytes },
    Payout { validators: Vec<Address>, id: Bytes,  receipt: Receipt, sigs: Vec<Signature>},
}

fn verify_payout_amount() -> Result<u128, Error> {
    // let mut remainder_capacity: u128 = 0;
    // let mut payout_capacity: u128 = 0;
    // let mut inputs_capacity: u128 = 0;

    let remainder_capacity = match load_cell_capacity(0, Source::Output) {
        Ok(rc) => (rc as u128),
        Err(err) => return Err(err.into()),
    };

    let payout_capacity = match load_cell_capacity(0, Source::Output) {
        Ok(pc) => (pc as u128),
        Err(err) => return Err(err.into()),
    };
    let inputs_capacity = match load_cell_capacity(0, Source::Input) {
        Ok(ic) => (ic as u128),
        Err(err) => return Err(err.into()),
    };

    // TODO: can there be an overflow here? if payout > remainder
    if inputs_capacity != remainder_capacity - payout_capacity {
        return Err(Error::InvalidPayoutAmount);
    };

    Ok(payout_capacity)
}

impl StateTransition {
    fn get() -> Result<Self, Error> {
        fn is_deploy() -> Result<bool, Error> {
            let my_hash = load_script_hash()?;
            return Ok(QueryIter::new(load_cell_type_hash, Source::Input)
                .filter(|option| option.map_or(false, |hash| hash == my_hash))
                .count() == 0);
        }

        let script_args: Bytes = load_script()?.args().raw_data();
        let validators = parse_validator_list_from_args(&*script_args)?;
        if validators.len() == 0 {
            return Err(Error::EmptyValidatorList);
        };
        let state_id: Bytes = get_state_id()?;
        debug!("validators: {:?}", validators);

        let isd = is_deploy()?;
        debug!("isd: {:?}", isd);
        if isd {
            return Ok(StateTransition::DeployBridge {
                validators: validators,
                id: state_id,
            })
        }

        // check state ID

        // check capacity
        verify_payout_amount();


        // load first witness
        let tx = load_transaction()?;
        let witness = tx.witnesses().get_unchecked(0);
        debug!("witness len: {:?}", witness);

        // read action byte
        let action_byte: u8 = (*witness.get_unchecked(0).as_slice())[0];

        // make receipt our own 💪
        let mut receipt: [u8; 128] = [0u8; 128];
        receipt.copy_from_slice(&witness.raw_data().slice(1..129));
        let sigs = Vec::new();

        match (*witness.get_unchecked(0).as_slice())[0] {
            0 => Ok(StateTransition::Payout{
                validators: validators,
                id: state_id,
                receipt: receipt,
                sigs: sigs
            }),
            _ => Err(Error::StateTransitionDoesNotExist),
        }
    }

    fn verify(&self) -> Result<(), Error> {
        match self {
            Self::DeployBridge { validators, id } => {
                // lock script on output0 should be anyone can spend
                let lock_code_hash = load_cell_lock(0, Source::Output)?.code_hash().raw_data();
                if *lock_code_hash != ANYONE_CAN_PAY_CODE_HASH[..] {
                    return Err(Error::WrongLockScript);
                }
                // type script on output0 should be our script
                let type_script_hash = load_cell_type_hash(0, Source::Output)?.unwrap();
                let script_hash = load_script_hash()?;

                if type_script_hash != script_hash {
                    return Err(Error::WrongTypeScript);
                }
                // data on output0 should be nothing
                let data = load_cell_data(0, Source::Output)?;

                if !data.len() == 0 {
                    return Err(Error::DataLengthNotZero);
                }
                // verify typescript args contains id and validators
                let type_script_0 = load_cell_type(0, Source::Output)?.unwrap();
                let type_script_args = type_script_0.args().raw_data();
                let validators_flat = Bytes::from(validators[..].concat());
                let id_and_validators = Bytes::from([&*id, &*validators_flat].concat());

                if id_and_validators != type_script_args {
                    return Err(Error::WrongStateId);
                }

                only_one_output_has_state_id()?;

                Ok(())
            },
            Self::Payout { validators:_, id:_, receipt, sigs:_ } => {

                debug!("isd: {:?}", receipt.len());
                Ok(())
            }
        }
    }
}

fn slice_to_array_20(slice: &[u8]) -> [u8; 20] {
    let mut array = [0u8; 20];
    for (&x, p) in slice.iter().zip(array.iter_mut()) {
        *p = x;
    }
    array
}

fn parse_validator_list_from_args(args: &[u8]) -> Result<Vec<Address>, Error> {
    // args consist of outpount + validator list
    // output has length of 36 bytes
    let val_args = &args[36..];
    // validator address
    if val_args.len() % ADDRESS_LEN != 0 {
        return Err(Error::InvalidArgsEncoding);
    }
    let mut validators = Vec::new();
    for i in 0..(val_args.len() / ADDRESS_LEN) {
        let ix = i * ADDRESS_LEN;
        validators.push(slice_to_array_20(&val_args[ix..ix + ADDRESS_LEN]));
    }
    Ok(validators)
}

fn get_state_id() -> Result<Bytes, Error> {
    let outpoint = load_input_out_point(0, Source::Input)?;
    let tx_hash: &[u8] = &*outpoint.tx_hash().raw_data();
    let index: &[u8] = &*outpoint.index().raw_data();
    Ok(Bytes::from([tx_hash, index].concat()))
}

// check there is always only one
fn only_one_output_has_state_id() -> Result<(), Error> {
    let my_hash = load_script_hash()?;
    let num = QueryIter::new(load_cell_type_hash, Source::Output)
        .filter(|option| option.map_or(false, |hash| hash == my_hash))
        .count();
    if num > 1 {
        return Err(Error::TooManyTypeOutputs);
    };
    Ok(())
}

fn main() -> Result<(), Error> {
    let state_transition = StateTransition::get()?;
    state_transition.verify()
}
