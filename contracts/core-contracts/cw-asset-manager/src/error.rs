use cosmwasm_std::{Addr, StdError};
use rlp::DecoderError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    //If a StdError is encountered and returned, it will be automatically converted into a ContractError using the #[from] attribute
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("RLP decoding error: {0}")]
    RlpDecodingError(#[from] DecoderError),

    #[error("Unacceptable token address: {address}")]
    InvalidToken { address: Addr },

    #[error("Sub call failed: {error}")]
    SubCallFailed { error: String },

    #[error("Token Deposit  Failed : {reason}")]
    DepositFailure { reason: String },

    #[error("Token Transfer Failed : {reason}")]
    TokenTransferFailure { reason: String },

    #[error("Deposit Revert Due to Xcall Failure : {account} : {token}")]
    RevertedDeposit { account: String, token: String },

    #[error("Xcall BTP Address is not found")]
    AddressNotFound,

    #[error("unknown method extracted while decoding rlp bytes")]
    UnknownMethod,

    #[error("Insufficient token balance")]
    InsufficientTokenBalance,

    #[error("only xcall service is allowed")]
    OnlyXcallService,

    #[error("xcall received data doesn't contained expected methods")]
    UnknownXcallDataReceived,
}
