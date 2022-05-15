use cosmwasm_std::StdError;
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("Insufficent funds (got {got:?}, needed {needed:?})")]
    InsufficientFunds { got: u128, needed: u128 },

    #[error("Incorrect password")]
    IncorrectPassword {},
}
