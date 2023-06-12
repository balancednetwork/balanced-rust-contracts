use cosmwasm_schema::cw_serde;

//structs to create XCALL messages
#[cw_serde]
pub struct WithdrawRequest {
    pub token_address: String,
    pub from_address: String,
    pub amount: u128,
}

#[cw_serde]
pub struct Deposit {
    pub token_address: String,
    pub from_address: String,
    pub to_address: String,
    pub amount: u128,
    pub data: Vec<u8>,
}

#[cw_serde]
pub struct DepositRevert {
    pub caller: String,
    pub amount: u128,
}

#[cw_serde]
pub struct WithdrawTo {
    pub token_address: String,
    pub user_address: String,
    pub amount: u128,
}

#[cw_serde]
pub enum ExecuteMsg {
    //executor address can be extracted at processing side
    Deposit { token_address: String, amount: u128 },

    WithdrawRequest { token_address: String, amount: u128 },
}

#[cw_serde]
pub enum XcallMsg {
    HandleCallMsg {
        deposit_revert: DepositRevert,
        withdraw_to: WithdrawTo,
    },

    SendCallMsg {
        deposit_from: Deposit,
        withdraw_req_from: WithdrawRequest,
    },
}
