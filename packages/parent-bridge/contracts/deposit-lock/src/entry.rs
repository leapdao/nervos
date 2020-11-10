
// Import from `core` instead of from `std` since we are in no-std mode
use core::result::Result;

use ckb_std::{
    high_level::{load_script, load_cell_lock_hash, load_cell_type_hash},
    ckb_types::{bytes::Bytes, prelude::*},
    ckb_constants::Source,
};

use crate::error::Error;

pub fn main() -> Result<(), Error> {

    let script = load_script()?;
    let args: Bytes = script.args().unpack();

    if args.len() != 64 {
        return Err(Error::WrongScriptArgsLength);
    }

    let allowed_lock_hash = args.slice(0..32);
    let allowed_type_hash = args.slice(32..64);

    let input0_lock_hash = load_cell_lock_hash(0, Source::Input)?;
    let input0_type_hash = load_cell_type_hash(0, Source::Input)?;

    let is_correct_type_hash = input0_type_hash.map_or(false, |h| allowed_type_hash == Bytes::from(&h[..]));
    let is_correct_lock_hash = Bytes::from(&input0_lock_hash[..]) == allowed_lock_hash;
    
    if !(is_correct_lock_hash || is_correct_type_hash) {
        return Err(Error::MissingCorrectTypeOrLockScript);
    }

    Ok(())
}

