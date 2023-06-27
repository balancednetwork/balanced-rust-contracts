

use rlp::{Encodable,RlpStream};
use cosmwasm_schema::cw_serde;


#[cw_serde]
pub struct Deposit {
    pub token_address: String,
    pub from: String,
    pub to: String,
    pub amount: u128,
}
#[cw_serde]
pub struct WithdrawRequest {
    pub token_address: String,
    pub from: String,
    pub amount: u128,
}
#[cw_serde]
pub struct DepositRevert {
    pub caller: String,
    pub amount: u128,
}

impl Encodable for Deposit {
    //specify the encoding logic for struct's fields so that rlp_bytes() can alo use
    fn rlp_append(&self, s: &mut RlpStream) {
        //append struct's each field to stream object
        let method = "Deposit".to_string();
        s.begin_list(5)
        .append(&method)
            .append(&self.token_address)
            .append(&self.from)
            .append(&self.to)
            .append(&self.amount);
    }
}




impl Encodable for WithdrawRequest {
    fn rlp_append(&self, s: &mut RlpStream) {
        let method = "Withdraw".to_string();
        s.begin_list(4)
            .append(&method)
            .append(&self.token_address)
            .append(&self.from)
            .append(&self.amount);
    }
}

impl Encodable for DepositRevert {
    fn rlp_append(&self, s: &mut RlpStream) {
        let method = "DepositRevert".to_string();
        s.begin_list(3)
            .append(&method)
            .append(&self.caller)
            .append(&self.amount);
    }
}





#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::Addr;

    #[test]
    fn test_encode(){
        let token = Addr::unchecked("token").to_string();
        let from = Addr::unchecked("from").to_string();
        let to = Addr::unchecked("to").to_string();


        let deposit = Deposit {
           token_address: token.clone(),
           from: from.clone(),
           to: to,
           amount: 1000,
        };


        let withdraw_req = WithdrawRequest {
           token_address: token,
           from: from.clone(),
           amount: 1000
        };


        let deposit_revert = DepositRevert {
            caller: from,
            amount:100
        };

    //use rlp bytes
    //internally relies on rlp_append to perform the actual encoding(you can check bro !)
    let encoded_deposit = deposit.rlp_bytes();
    let encoded_withdraw = withdraw_req.rlp_bytes();
    let encode_deposit_revert = deposit_revert.rlp_bytes();
        
    // Use rlp_append
    let mut stream = RlpStream::new();
    deposit.rlp_append(&mut stream);
    let encoded_append = stream.out();

    //ensuring both methods generates identical encoded bytes 
    assert_eq!(encoded_deposit,encoded_append);
    
    //checking if encoded structs are different
    assert_ne!(encoded_deposit,encode_deposit_revert);
    assert_ne!(encoded_withdraw,encode_deposit_revert);
    assert_ne!(encoded_deposit,encoded_withdraw);
    

    }



}