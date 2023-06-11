use cosmwasm_schema::cw_serde;


//External Methods
#[cw_serde]
pub struct  Deposit {
pub token_address: String,
pub from_address: String,
pub to_address: String,
pub amount: u128,
pub data : Vec<u8>,
}


#[cw_serde]
pub struct WithdrawRequest {
    token_address: String,
        from_address:  String,
        amount: u128,
} 



//Xcall Methods
#[cw_serde]
pub struct  DepositTo {
pub token_address: String,
pub from_address: String,
pub to_address : String,
pub amount: u128,
pub data: Vec<u8>,
}


pub struct DepositRevert {
    pub caller: String,
    pub amount: Uint128,
}


pub struct WithdrawTo {
    token_address: String,
        user_address:  String,
        amount: u128,
}



#[cw_serde]
pub enum ExecuteMsg {
   deposit(Deposit),
   withdraw_request(WithdrawRequest),
}


#[cw_serde]
pub enum XcallMsg {
    HandlecallMsg {
        deposit_revert : DepositRevert,
        withdraw_to : WithdrawTo,

    },
    
    SendcallMsg {
        deposit_from : Deposit,
        withdraw_req_from : WithdrawRequest,
        
    }
}











