use cosmwasm_std::{StdError, Addr, Uint128};
use thiserror::Error;
use rlp::DecoderError;

#[derive(Error, Debug,PartialEq)]
pub enum ContractError {
    //If a StdError is encountered and returned, it will be automatically converted into a ContractError using the #[from] attribute
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("RLP decoding error: {0}")]
    RlpDecodingError(#[from] DecoderError),

    #[error("Unacceptable token address: {address}")]
    InvalidToken {
        address: Addr,
    },

    #[error("Subcall failed: {error}")]
    SubcallFailed{
        error: String,
    },

    #[error("Token Deposit  Failed : {reason}")]
    DepositFailure {
        reason: String,
    },

    #[error("Token Transfer Failed : {reason}")]
    TokenTransferFailure {
        reason: String,
    },

    #[error("Deposit Revert Due to Xcall Failure : {account} : {token}")]
    RevertedDeposit {
        account: String,
        token: String,
    },



    #[error("Xcall BTP Address is not found")]
    AddressNotFound,
    
    #[error("unknown method extracted while decoding rlp bytes")]
    UnknownMethod,

    #[error("Insufficinet token balance : address-{token} amount-{current_balance}")]
    InsufficentTokenBalance {
        token: String,
        current_balance: Uint128,
    },

    #[error("only xcall service is allowed")]
    OnlyXcallService,

    
}



