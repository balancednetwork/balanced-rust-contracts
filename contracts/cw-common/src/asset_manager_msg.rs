use cosmwasm_schema::{cw_serde,QueryResponses};
use cosmwasm_std::Uint128;



#[cw_serde]
pub struct InstantiateMsg {
    pub cw20_whitelist: Vec<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    //executor address can be extracted at processing side
    Deposit { token_address: String, amount: Uint128 },

    WithdrawRequest { token_address: String, amount: Uint128 },

    ConfigureXcall {
        source_xcall: String,
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




















