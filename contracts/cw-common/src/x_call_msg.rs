use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum XCallQuery {
    #[returns(String)]
    GetNetworkAddress {},
}

//TODO: Use the ibc-integration/xcallmsg and xcall contract from ibc
#[cw_serde]
pub enum XCallExecuteMsg {
    SetDefaultConnection {
        nid: String,
        address: Addr,
    },

    /**
     * to send a call message to the contract on the destination chain.
     * @param _to The network address of the callee on the destination chain
     * @param _data The calldata specific to the target contract
     * @param _rollback (Optional) The data for restoring the caller state when an error occurred
     * @param sources  (Optional) The contracts that will be used to send the message
     * @param destinations (Optional) The addresses of the contracts that xcall will expect the message from.
     *
     */
    SendCallMessage {
        to: String,
        data: Vec<u8>,
        rollback: Option<Vec<u8>>,
        sources: Option<Vec<String>>,
        destinations: Option<Vec<String>>,
    },

    HandleMessage {
        from: String, //NetId
        sn: Option<i64>,
        msg: Vec<u8>,
    },

    ExecuteCall {
        request_id: u128,
        data: Vec<u8>,
    },
}
