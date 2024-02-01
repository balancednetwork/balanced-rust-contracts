use cosmwasm_std::StdError;
use cw_ibc_rlp_lib::rlp::DecoderError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    //If a StdError is encountered and returned, it will be automatically converted into a ContractError using the #[from] attribute
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Only proposer is allowed")]
    OnlyProposer,

    #[error("Only governance is allowed")]
    OnlyGovernance,

    #[error("Only xcall is allowed")]
    OnlyXCall,

    #[error("Invalid Method")]
    InvalidMethod,

    #[error("Network has not been configured")]
    NetworkNotConfigured,

    #[error("Invalid protocols")]
    InvalidProtocol,

    #[error("Rlp Error: {error}")]
    DecoderError { error: DecoderError },
}

impl From<DecoderError> for ContractError {
    fn from(err: DecoderError) -> Self {
        ContractError::DecoderError { error: err }
    }
}
