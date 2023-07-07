use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub cw20_whitelist: Vec<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    //executor address can be extracted at processing side
    Deposit {
        //TODO: normal archway validation
        token_address: String,
        amount: Uint128,
    },

    //TODO: introduce deposit transfer,
    // to field: network address(validation) to receive the (can be loans, another user address) (optional) defaults to caller
    // data field: depending upon the to address (optional)
    WithdrawRequest {
        token_address: String,
        amount: Uint128,
    },

    ConfigureXcall {
        source_xcall: String,
        //TODO: rename to destination asset manager
        destination_contract: String,
    },

    HandleCallMessage {
        from: String,
        data: Vec<u8>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
