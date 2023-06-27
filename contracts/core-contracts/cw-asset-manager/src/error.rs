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

    #[error("Token Transfer Failed : {reason}")]
    TokenTransferFailure {
        reason: String,
    },

    #[error("Xcall BTP Address is not found")]
    AddressNotFound,
    
    #[error("unknown method extracted while decoding rlp bytes")]
    UnknownMethod,

   
    
}



