use cosmwasm_std::{StdError, Addr};
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

    #[error("Insufficinet token balance")]
    InsufficentTokenBalance,
    

    #[error("only xcall service is allowed")]
    OnlyXcallService,

    #[error("only contract owner is allowed")]
    OnlyOwner,

    #[error("xcall recieved data doesn't contained expected methods")]
    UnknownXcallDataRecieved,

    
}



