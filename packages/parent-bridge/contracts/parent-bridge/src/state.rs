use ckb_std::ckb_types::packed::Transaction;
use ckb_std::high_level::load_transaction;

use crate::error::Error;

pub trait State: Eq {
    type Action;
    type Effect: Eq;

    fn next(&self, action: Self::Action) -> Result<(Self, Self::Effect), Error>
    where
        Self: Sized,
        Self::Action: Sized,
        Self::Effect: Sized;
    fn get_init_state(tx: &Transaction) -> Result<Self, Error>
    where
        Self: Sized;
    fn get_end_state(tx: &Transaction) -> Result<Self, Error>
    where
        Self: Sized;
    fn get_action(tx: &Transaction) -> Result<Self::Action, Error>
    where
        Self::Action: Sized;
    fn get_effect(tx: &Transaction) -> Result<Self::Effect, Error>
    where
        Self::Effect: Sized;
}

fn verify_id(tx: &Transaction) -> Result<(), Error> {
    Ok(())
}

fn verify<S: State>() -> Result<(), Error> {
    
    let tx = load_transaction()?;

    verify_id(&tx)?;
    
    let init_state = S::get_init_state(&tx)?;
    let end_state = S::get_end_state(&tx)?;
    let action = S::get_action(&tx)?;
    let effect = S::get_effect(&tx)?;

    let (expected_end_state, expected_effect) = S::next(&init_state, action)?;

    // figure out how to do proper errors here
    if expected_end_state != end_state {
        return Err(Error::IndexOutOfBound);
    }
    if expected_effect != effect {
        return Err(Error::IndexOutOfBound);
    }

    Ok(())
}
