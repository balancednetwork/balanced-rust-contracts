use cosmwasm_schema::cw_serde;


//External Methods
#[cw_serde]
pub struct  Deposit {
pub token_address: String,
pub from_address: String,
pub amount: u128,
}






    

#[cw_serde]
pub struct WithdrawTo {
    token_address: String, // Native address as string
        user_address:  String,// Native address as string
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











#[cw_serde]
pub enum ExecuteMsg {
    Deposit {},
    WithdrawTo {},
}





#[cw_serde]
pub enum XcallMsg {
    HandleCallMsg {
        deposit_revert : DepositRevert,

    },
    
    SendCallMsg {
        deposit_to : DepositTo,
        withdraw : withdraw,
    }
}











