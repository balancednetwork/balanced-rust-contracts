use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

//structs to create XCALL messages data structure



//outgoing xcall messages
#[cw_serde]
pub struct WithdrawRequest {
    pub token_address: String,
    pub from_address: String,
    pub amount: Uint128,
}

impl WithdrawRequest {
    pub fn new(token_addr: String,from: String,token_amount: Uint128) -> Self {
      Self {
         token_address: token_addr, 
         from_address: from, 
         amount: token_amount,
        }
    }
}

#[cw_serde]
pub struct Deposit {
    pub token_address: String,
    pub from_address: String,
    pub to_address: String,
    pub amount: Uint128,
    pub data: Vec<u8>,
}

impl Deposit {
    pub fn new(
        token_address: String,
        from_address: String,
        to_address: String,
        amount: Uint128,
        data: Vec<u8>,
    ) -> Self {
        Self {
            token_address,
            from_address,
            to_address,
            amount,
            data,
        }
    }
}






//incoming messages
#[cw_serde]
pub struct DepositRevert {
    pub caller: String,
    pub amount: Uint128,
}



#[cw_serde]
pub struct WithdrawTo {
    pub token_address: String,
    pub user_address: String,
    pub amount: Uint128,
}

#[cw_serde]
pub enum ExecuteMsg {
    //executor address can be extracted at processing side
    Deposit { token_address: String, amount: Uint128 },

    WithdrawRequest { token_address: String, amount: Uint128 },
}




















