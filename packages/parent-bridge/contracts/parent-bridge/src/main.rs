#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]

// Import from `core` instead of from `std` since we are in no-std mode
use core::result::Result;

// Import heap related library from `alloc`
// https://doc.rust-lang.org/alloc/index.html
use alloc::{vec::Vec};

use ckb_std::{
    ckb_constants::Source,
    ckb_types::{bytes::Bytes},
    debug, default_alloc, entry,
    error::SysError,
    high_level::{
        load_cell_data, load_cell_lock, load_cell_type_hash, load_script, load_script_hash,
            },
    syscalls::load_witness,
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

/// Error
#[repr(i8)]
enum Error {
    IndexOutOfBound = 1,
    ItemMissing = 2,
    LengthNotEnough = 3,
    Encoding = 4,
    StateTransitionDoesNotExist = 5,
    WrongValidatorListLength = 6,
    WrongLockScript = 7,
    WrongTypeScript = 8,
    DataLengthNotZero = 9,
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

type Address = [u8; 20];

enum StateTransition {
    DeployBridge { validators: Vec<Address> },
}

impl StateTransition {
    fn verify(&self) -> Result<(), Error> {
        match self {
            Self::DeployBridge { validators: _ } => {
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
    if args.len() % 20 != 0 {
        return Err(Error::WrongValidatorListLength);
    }
    let mut validators = Vec::new();
    for i in 0..(args.len() / 20) {
        validators.push(slice_to_array_20(&args[i..i + 20]));
    }
    Ok(validators)
}

fn get_state_transition() -> Result<StateTransition, Error> {
    let mut wit_buf: [u8; 1] = [0; 1];
    load_witness(&mut wit_buf, 0, 0, Source::Input)?;
    match wit_buf[0] {
        0 => {
            let script_args: Bytes = load_script()?.args().raw_data();
            let validators = parse_validator_list_from_args(&*script_args)?;
            debug!("validators: {:?}", validators);
            Ok(StateTransition::DeployBridge {
                validators: validators,
            })
        }
        _ => Err(Error::StateTransitionDoesNotExist),
    }
}

fn main() -> Result<(), Error> {
    let state_transition = get_state_transition()?;
    state_transition.verify()
}
