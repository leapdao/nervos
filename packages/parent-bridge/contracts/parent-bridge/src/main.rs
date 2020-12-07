#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]

// Import from `core` instead of from `std` since we are in no-std mode
use core::result::Result;

mod code_hashes;
use code_hashes::{CODE_HASH_DEPOSIT_LOCK, CODE_HASH_ANYONE_CAN_SPEND};

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
        load_script, load_script_hash, QueryIter, load_transaction, load_cell_capacity, load_input,
    },
};
use k256::{
    ecdsa::{recoverable},
};
use sha3::{Digest, Keccak256};
use hex::encode;
use hex::FromHex;
use core::convert::TryFrom;

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

const ADDRESS_LEN: usize = 20;

/// Error
#[repr(i8)]
enum Error {
    IndexOutOfBound = 1,
    ItemMissing = 2,
    LengthNotEnough = 3,
    Encoding = 4,
    // Add customized errors here...
    StateTransitionDoesNotExist = 5,
    InvalidArgsEncoding = 6,
    WrongLockScript = 7,
    WrongTypeScript = 8,
    DataLengthNotZero = 9,
    WrongStateId = 10,
    TooManyTypeOutputs = 11,
    EmptyValidatorList = 12,
    InvalidPayoutAmount = 13,
    InvalidWitnessEncoding = 14,
    InconsistentStateId = 15,
    // Add customized errors here...
    DepositCapacityComputedIncorrectly = 16,
    DepositsShouldNotChangeData = 17,
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
    DeployBridge { validators: Vec<Address>, id: Bytes , trustee: Address},
    Payout { validators: Vec<Address>, id: Bytes,  receipt: Receipt, sigs: Vec<Signature>, amount: u128},
    CollectDeposits {
        total: u64,
        cap_before: u64,
        cap_after: u64,
        data_before: Vec<u8>,
        data_after: Vec<u8>,
    },
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
                .count()
                == 0);
        }

        let script_args: Bytes = load_script()?.args().raw_data();
        let validators = parse_validator_list_from_args(&*script_args)?;
        if validators.len() == 0 {
            return Err(Error::EmptyValidatorList);
        };
        let state_id: Bytes = get_state_id()?;
        debug!("validators: {:?}", validators);
        let trustee = parse_trustee_from_args(&*script_args)?;

        // check state ID
        only_one_output_has_state_id()?;

        let isd = is_deploy()?;
        if isd {
            return Ok(StateTransition::DeployBridge {
                validators: validators,
                id: state_id,
                trustee: trustee,
            });
        }
        // check capacity
        //let amount: u128 = verify_payout_amount()?;

        let amount: u128 = 0;
        // load first witness
        let tx = load_transaction()?;
        let witness = tx.witnesses().get_unchecked(0);


        // read action byte
        let action_byte: u8 = (*witness.get_unchecked(0).as_slice())[0];

        // distinguished based on first byte of witness
        match action_byte {
            0 => {
                //check for correct Encoding of Witness
                if witness.len() >= 194 && (witness.len()-129) % 65 != 0 {
                    return Err(Error::InvalidWitnessEncoding);
                }
                // make receipt our own 💪
                let mut receipt: [u8; 128] = [0u8; 128];
                receipt.copy_from_slice(&witness.raw_data().slice(1..129));
                let mut sigs = Vec::new();
                //calculate vector length of signatures
                let signatures_vector_length = (witness.len()-129)/65;

                for x in 0..signatures_vector_length {
                    let mut temp_sig: [u8; 65] = [0u8; 65];
                    temp_sig.copy_from_slice(&witness.raw_data().slice(129+x*65 .. 129+(x+1)*65));
                    sigs.push(temp_sig);
                }
                Ok(StateTransition::Payout{
                validators: validators,
                id: state_id,
                receipt: receipt,
                sigs: sigs,
                amount: amount
            })
            },
            1 => {
                let bridge_cap_before = load_cell_capacity(0, Source::Input)?;
                let bridge_cap_after = load_cell_capacity(0, Source::Output)?;
                let total_deposit_capacity = QueryIter::new(load_cell_lock, Source::Input)
                    .zip(QueryIter::new(load_cell_capacity, Source::Input))
                    .filter(|(script, _)| {
                        *script.code_hash().raw_data() == CODE_HASH_DEPOSIT_LOCK[..]
                    })
                    .map(|(_, cap)| cap)
                    .sum();
                let data_before = load_cell_data(0, Source::Input)?;
                let data_after = load_cell_data(0, Source::Output)?;
                Ok(Self::CollectDeposits {
                    total: total_deposit_capacity,
                    cap_before: bridge_cap_before,
                    cap_after: bridge_cap_after,
                    data_before: data_before,
                    data_after: data_after,
                })
            },
            _ => Err(Error::StateTransitionDoesNotExist),
        }
    }

    fn verify(&self) -> Result<(), Error> {
        match self {
            Self::DeployBridge { validators, id ,trustee} => {
                // lock script on output0 should be anyone can spend
                let lock_code_hash = load_cell_lock(0, Source::Output)?.code_hash().raw_data();
                if *lock_code_hash != CODE_HASH_ANYONE_CAN_SPEND[..] {
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

                if data.len() != 0 {
                    return Err(Error::DataLengthNotZero);
                }

                // verify typescript args contains id and trustee and validators
                let type_script_0 = load_cell_type(0, Source::Output)?.unwrap();
                let type_script_args = type_script_0.args().raw_data();
                let validators_flat = Bytes::from(validators[..].concat());
                let id_and_trustee_and_validators = Bytes::from([&*id, &*Bytes::from(&trustee[..]), &*validators_flat].concat());

                if id_and_trustee_and_validators != type_script_args {
                    return Err(Error::WrongStateId);
                }


                Ok(())
            },
            Self::Payout { validators, id, receipt, sigs, amount } => {
                //let mut signerAddrs: = Vec::new();
                for i in 0..(sigs.len()) {
                    //let hash = Keccak256::digest(&receipt[..]);
                    let hash = Keccak256::new().chain(&receipt[..]);
                    //let signature: = Signature.new(Signature::from(sigs[i].slice(0..64)), sigs[i]);
                    //let sig: recoverable::Signature = recoverable::Signature::into(&sigs[i][..]);
                    let sig: recoverable::Signature = recoverable::Signature::try_from(&sigs[i][..]).unwrap();
                    let recovered_key = sig.recover_verify_key_from_digest(hash).unwrap();
                    // signerAddrs[i] =
                    //debug!("key: 0x{:?}", hex::encode(&recovered_key.to_bytes()[0..32]));
                }
                let hash = Keccak256::digest(&Bytes::from(Vec::from_hex("00000000000000000000000000000000000000000000000000000000000000011122334411223344112233441122334411223344112233441122334411223344000000000000000000000000112233445566778899001122334455667788990000000000000000000000000000000000000000000000000000000000000004D2").unwrap()));
                debug!("Hash2: {:?}", encode(hash));

                debug!("debug: {:?}, id: {:?}, receipt: {:?}, sigs: {:?}, amount: {:?}", validators, id, receipt.len(), sigs.len(), amount);
                Ok(())
            }
            Self::CollectDeposits {
                total,
                cap_before,
                cap_after,
                data_before,
                data_after,
            } => {
                verify_state_id()?;
                if *cap_after != total + cap_before {
                    return Err(Error::DepositCapacityComputedIncorrectly);
                }
                if data_before != data_after {
                    return Err(Error::DepositsShouldNotChangeData);
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
    // args consist of outpount + validator list
    // output has length of 36 bytes + trustee is 20 bytes
    let val_args = &args[56..];
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

fn parse_trustee_from_args(args: &[u8]) -> Result<Address, Error> {
    Ok(slice_to_array_20(&args[36..56]))
}

fn get_state_id() -> Result<Bytes, Error> {
    let outpoint = load_input_out_point(0, Source::Input)?;
    let tx_hash: &[u8] = &*outpoint.tx_hash().raw_data();
    let index: &[u8] = &*outpoint.index().raw_data();
    Ok(Bytes::from([tx_hash, index].concat()))
}

fn verify_state_id() -> Result<(), Error> {
    let num_outputs = QueryIter::new(load_input, Source::GroupOutput).count();

    if num_outputs > 1 {
        return Err(Error::TooManyTypeOutputs);
    }

    Ok(())
}

// check there is always only one
fn only_one_output_has_state_id() -> Result<(), Error> {
    //load currently executed script, in this case Bridge type script
    let my_hash = load_script_hash()?;
    //check how many times identical script appears in Outputs
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
