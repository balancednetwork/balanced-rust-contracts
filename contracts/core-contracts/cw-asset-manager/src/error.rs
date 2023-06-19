use cosmwasm_std::{StdError, Addr};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    //If a StdError is encountered and returned, it will be automatically converted into a ContractError using the #[from] attribute
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unacceptable token address: {address}")]
    InvalidToken {
        address: Addr,
    },


    #[error("Unable to parse data from response")]
    ErrorInParsing{},

    


    
}
