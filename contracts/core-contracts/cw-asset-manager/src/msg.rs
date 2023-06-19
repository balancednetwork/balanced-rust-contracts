use cosmwasm_schema::{cw_serde, QueryResponses};


#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub token_addr: String,
}

// #[cw_serde]
// pub enum ExecuteMsg {

// }

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(DepositlistResponse)]
    get_deposits_list {},
}


#[cw_serde]
pub struct DepositlistResponse {

}


#[cw_serde]
pub enum XcallMessage {
    SendCallMessage {
        to: String,
        data: Vec<u8>,
        rollback: Option<Vec<u8>>,
    },
}