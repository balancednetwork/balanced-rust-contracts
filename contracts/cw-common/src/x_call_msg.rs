use crate::network_address::{NetId, NetworkAddress};
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
pub enum XCallMsg {
    SetDefaultConnection {
        nid: NetId,
        address: Addr,
    },

    SendCallMessage {
        to: NetworkAddress,
        data: Vec<u8>,
        rollback: Option<Vec<u8>>,
        sources: Option<Vec<String>>,
        destinations: Option<Vec<String>>,
    },
    HandleMessage {
        from: NetId,
        sn: Option<i64>,
        msg: Vec<u8>,
    },

    ExecuteCall {
        request_id: u128,
        data: Vec<u8>,
    },

    ExecuteRollback {
        sequence_no: u128,
    },

    TestHandleCallMessage {
        from: String,
        data: Vec<u8>,
        hub_token: String,
    },
}
