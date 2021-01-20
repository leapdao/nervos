#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]

// Import from `core` instead of from `std` since we are in no-std mode
use core::result::Result;

mod code_hashes;
use code_hashes::{CODE_HASH_DEPOSIT_LOCK, CODE_HASH_ANYONE_CAN_SPEND, CODE_HASH_AUDIT_DELAY};

// Import heap related library from `alloc`
// https://doc.rust-lang.org/alloc/index.html
use alloc::vec::Vec;

use ckb_std::{
    debug,
    ckb_constants::Source,
    ckb_types::bytes::Bytes,
    default_alloc, entry,
    error::SysError,
    high_level::{
        load_cell_data, load_cell_lock, load_cell_type, load_cell_type_hash, load_input_out_point,
        load_script, load_script_hash, QueryIter, load_transaction, load_cell_capacity, load_input,
        load_cell_lock_hash,
    },
};
use core::convert::TryFrom;
use core::convert::TryInto;
use elliptic_curve::sec1::ToEncodedPoint;
use k256::ecdsa::{recoverable};
use sha3::{Digest, Keccak256};

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

// TODO: this parameter should be moved to bridge args eventually
const TIMEOUT: u64 = 100;
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
    WrongScriptArgsLength = 13,
    InvalidWitnessEncoding = 14,
    InvalidWithdrawalCapacity = 15,
    DepositCapacityComputedIncorrectly = 16,
    DepositsShouldNotChangeData = 17,
    NotSignedByTrustee = 18,
    BridgeWasNotDissolved = 19,
    LeftoverCapacity = 20,
    UnknownReceiptSigner = 21,
    SignatureQuorumNotMet = 22,
    WithdrawalCapacityComputedIncorrectly = 23,
    DataUpdatedIncorrectly = 24, 
    WrongTrusteeInPayout = 25,
    WrongPayoutDestination = 26,
    WrongTimeout = 27,
    ReceiptAlreadyUsed = 28,
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
type Hash = [u8;32];
type Receipt = [u8; 128];
type Signature = [u8; 65];


enum StateTransition {
    DeployBridge { validators: Vec<Address>, id: Bytes , trustee: Hash},
    Payout {
        validators: Vec<Address>,
        receipt: Receipt,
        sigs: Vec<Signature>,
        cap_before: u64,
        cap_after: u64,
        data_before: Vec<u8>,
        data_after: Vec<u8>,
        trustee: [u8; 32],
    },
    CollectDeposits {
        total: u64,
        cap_before: u64,
        cap_after: u64,
        data_before: Vec<u8>,
        data_after: Vec<u8>,
    },
    HaltAndDissolve { trustee: Hash},
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
        // debug!("validators: {:?}", validators);
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

        // load first witness
        let tx = load_transaction()?;
        let witness = tx.witnesses().get_unchecked(0);

        // read action byte
        let action_byte: u8 = (*witness.get_unchecked(0).as_slice())[0];
        let bridge_cap_before = load_cell_capacity(0, Source::Input)?;
        let bridge_cap_after = load_cell_capacity(0, Source::Output)?;
        let data_before = load_cell_data(0, Source::Input)?;
        let data_after = load_cell_data(0, Source::Output)?;

        // distinguished based on first byte of witness
        match action_byte {
            // prepare and call payout
            0 => {
                //check for correct Encoding of Witness
                if witness.len() >= 194 && (witness.len() - 129) % 65 != 0 {
                    return Err(Error::InvalidWitnessEncoding);
                }
                // make receipt our own 💪
                let mut receipt: [u8; 128] = [0u8; 128];
                receipt.copy_from_slice(&witness.raw_data().slice(1..129));
                let mut sigs = Vec::new();
                //calculate vector length of signatures
                let signatures_vector_length = (witness.len() - 129) / 65;

                for x in 0..signatures_vector_length {
                    let mut temp_sig: [u8; 65] = [0u8; 65];
                    temp_sig.copy_from_slice(
                        &witness.raw_data().slice(129 + x * 65..129 + (x + 1) * 65),
                    );
                    sigs.push(temp_sig);
                }
                Ok(StateTransition::Payout {
                    validators: validators,
                    receipt: receipt,
                    sigs: sigs,
                    cap_before: bridge_cap_before,
                    cap_after: bridge_cap_after,
                    data_before: data_before,
                    data_after: data_after,
                    trustee: trustee,
                })
            }
            // prepare and call "collect deposits"
            1 => {
                let total_deposit_capacity = QueryIter::new(load_cell_lock, Source::Input)
                    .zip(QueryIter::new(load_cell_capacity, Source::Input))
                    .filter(|(script, _)| {
                        *script.code_hash().raw_data() == CODE_HASH_DEPOSIT_LOCK[..]
                    })
                    .map(|(_, cap)| cap)
                    .sum();
                Ok(Self::CollectDeposits {
                    total: total_deposit_capacity,
                    cap_before: bridge_cap_before,
                    cap_after: bridge_cap_after,
                    data_before: data_before,
                    data_after: data_after,
                })
            },
            2 => {
                Ok(StateTransition::HaltAndDissolve{
                    trustee: trustee,
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
            }
            Self::Payout {
                validators,
                receipt,
                sigs,
                cap_before,
                cap_after,
                data_before,
                data_after,
                trustee,
            } => {
                let hash = Keccak256::digest(&receipt[..]);
                let mut quorum = Vec::new();
                // init verification vector
                for i in 0..(validators.len()) {
                    quorum.push(false);
                }
                // recover all signers
                for i in 0..(sigs.len()) {
                    let preamble: &[u8] = b"\x19Ethereum Signed Message:\n128";
                    let sig: recoverable::Signature =
                        recoverable::Signature::try_from(&sigs[i][..]).unwrap();
                    let recovered_key = sig.recover_verify_key([preamble, &receipt[..]].concat().as_slice()).unwrap();
                    let mut addr: [u8; 20] = [0u8; 20];
                    addr.copy_from_slice(
                        &Keccak256::digest(&recovered_key.to_encoded_point(false).as_bytes()[1..65])[12..],
                    );
                    let pos = get_position(addr, validators)?;
                    quorum[pos] = true;
                }
                // determine quorum
                let mut sig_count = 0;
                for i in 0..(validators.len()) {
                    if quorum[i] {
                        sig_count += 1;
                    }
                }
                if sig_count < validators.len() * 2 / 3 {
                    return Err(Error::SignatureQuorumNotMet);
                }
                // check capacity
                let mut amount_array: [u8; 8] = [0u8; 8];
                amount_array.copy_from_slice(&receipt[56..64]);
                let amount = u64::from_be_bytes(amount_array);
                if *cap_after != cap_before - amount {
                    return Err(Error::WithdrawalCapacityComputedIncorrectly);
                }
                // check payout output
                let payout_cap = load_cell_capacity(1, Source::Output)?;
                if payout_cap != amount {
                    return Err(Error::InvalidWithdrawalCapacity);
                }
                let lock_code_hash = load_cell_lock(1, Source::Output)?.code_hash().raw_data();
                if *lock_code_hash != CODE_HASH_AUDIT_DELAY[..] {
                    return Err(Error::WrongLockScript);
                }
                let lock_args = load_cell_lock(1, Source::Output)?.args().raw_data();
                if lock_args.len() != 72 {
                    return Err(Error::WrongScriptArgsLength);
                }
                let trustee_lock_hash = lock_args.slice(0..32);
                if *trustee_lock_hash != trustee[..] {
                    return Err(Error::WrongTrusteeInPayout);
                }
                let owner_lock_hash = lock_args.slice(32..64);
                if *owner_lock_hash != receipt[64..96] {
                    return Err(Error::WrongPayoutDestination);
                }
                let timeout_array : [u8; 8] = (&*lock_args.slice(64..72)).try_into().expect("could not parse timeout");
                let timeout = u64::from_be_bytes(timeout_array);
                if timeout != TIMEOUT {
                    return Err(Error::WrongTimeout);
                }

                let expected_data = [data_before, &hash[..]].concat();
                if data_after != &expected_data {
                    return Err(Error::DataUpdatedIncorrectly);
                }

                let used_hashes = parse_data(data_before);
                if used_hashes.iter().any(|h| h == &hash[..].to_vec()) {
                    return Err(Error::ReceiptAlreadyUsed);
                }
                
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
            Self::HaltAndDissolve {trustee} => {
                //Is trustee signer of any input?
                let trustee_signed = QueryIter::new(load_cell_lock_hash, Source::Input)
                    .filter(|hash| hash == trustee)
                    .count()
                    > 0;
                if !trustee_signed {
                    return Err(Error::NotSignedByTrustee);
                }
                //Check there is no bridge in the outputs
                let my_hash = load_script_hash()?;
                let is_bridge_in_outputs = QueryIter::new(load_cell_type_hash, Source::Output)
                    .filter(|option| option.map_or(false, |hash| hash == my_hash))
                    .count()
                    > 0;
                if is_bridge_in_outputs {
                    return Err(Error::BridgeWasNotDissolved);
                }
                //Check all capacity is spent
                let bridge_cap = load_cell_capacity(0, Source::GroupInput)?;
                let outputs_cap = QueryIter::new(load_cell_capacity, Source::Output)
                    .sum();
                if bridge_cap > outputs_cap {
                    return Err(Error::LeftoverCapacity);
                }
                Ok(())
            }
        }
    }
}

fn parse_data(data: &Vec<u8>) -> Vec<Vec<u8>> {
    let mut parsed = Vec::new();
    for i in 0..(data.len() / 32) {
        parsed.push(data[(i*32)..(i*32)+32].to_vec());
    }
    parsed
}

fn slice_to_array_20(slice: &[u8]) -> [u8; 20] {
    let mut array = [0u8; 20];
    for (&x, p) in slice.iter().zip(array.iter_mut()) {
        *p = x;
    }
    array
}

fn slice_to_array_32(slice: &[u8]) -> [u8; 32] {
    let mut array = [0u8; 32];
    for (&x, p) in slice.iter().zip(array.iter_mut()) {
        *p = x;
    }
    array
}

fn get_position(address: Address, vec: &Vec<Address>) -> Result<usize, Error> {
    for i in 0..vec.len() {
        if address == vec[i] {
            return Ok(i);
        }
    }
    return Err(Error::UnknownReceiptSigner); // todo: use proper error
}

fn parse_validator_list_from_args(args: &[u8]) -> Result<Vec<Address>, Error> {
    // args consist of outpount + validator list
    // output has length of 36 bytes + trustee is 32 bytes
    let val_args = &args[68..];
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

fn parse_trustee_from_args(args: &[u8]) -> Result<[u8;32], Error> {
    Ok(slice_to_array_32(&args[36..68]))
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
