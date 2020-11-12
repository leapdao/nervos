// Import from `core` instead of from `std` since we are in no-std mode
use core::result::Result;
use core::convert::TryInto;

// Import heap related library from `alloc`
// https://doc.rust-lang.org/alloc/index.html
use alloc::{vec, vec::Vec};

// Import CKB syscalls and structures
// https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/index.html
use ckb_std::{
    ckb_constants::Source,
    debug,
    high_level::{load_script, load_tx_hash, load_header, load_cell_lock_hash},
    ckb_types::{bytes::Bytes, prelude::*},
};

use crate::error::Error;

pub fn main() -> Result<(), Error> {
    let input0_lock_hash = load_cell_lock_hash(0, Source::Input)?;
    let script = load_script()?;
    let args: Bytes = script.args().unpack();

    if args.len() != 72 {
        return Err(Error::WrongScriptArgsLength);
    }

    let trustee_lock_hash = args.slice(0..32);
    let owner_lock_hash = args.slice(32..64);

    if Bytes::from(&input0_lock_hash[..]) == trustee_lock_hash {
        return Ok(());
    }

    let input_header = load_header(0, Source::GroupInput)?;
    let input_header_timestamp = input_header.raw().timestamp().unpack();
    let proof_header = load_header(1, Source::HeaderDep)?;
    let proof_header_timestamp = proof_header.raw().timestamp().unpack();
    let timeout = args.slice(64..72);
    let timeout_array: [u8; 8] = (&*timeout).try_into().unwrap();
    let timeout_num = u64::from_be_bytes(timeout_array);
    
    if proof_header_timestamp - input_header_timestamp < timeout_num {
        return Err(Error::NotEnoughTimePassed);
    }

    let is_correct_lock_hash = Bytes::from(&input0_lock_hash[..]) == owner_lock_hash;

    if !is_correct_lock_hash {
        return Err(Error::NotSpentWithOwnerInput);
    }
    
    Ok(())
}

