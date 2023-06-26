

use rlp::{Decodable, Encodable,DecoderError,RlpStream,Rlp};
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
        s.begin_list(4)
            .append(&self.token_address)
            .append(&self.from)
            .append(&self.to)
            .append(&self.amount);
    }
}



//implementing decodable trait just for testing
impl Decodable for Deposit {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {

        //check if the rlp is list and contains items
        if !rlp.is_list() || rlp.item_count()? != 4 {
            return Err(DecoderError::RlpIncorrectListLen);
        }

        Ok(Self {
            token_address: rlp.val_at(0)?,
            from: rlp.val_at(1)?,
            to: rlp.val_at(2)?,
            amount: rlp.val_at(3)?
        })
    }
}

impl Encodable for WithdrawRequest {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(3)
            .append(&self.token_address)
            .append(&self.from)
            .append(&self.amount);
    }
}

impl Encodable for DepositRevert {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(3)
            .append(&self.caller)
            .append(&self.amount);
    }
}





#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::Addr;

    #[test]
    fn test_encode_decode_deposit() {
        let token = Addr::unchecked("token");
        let from = Addr::unchecked("from");
        let to = Addr::unchecked("to");


        let deposit = Deposit {
           token_address: token.to_string(),
           from: from.to_string(),
           to: to.to_string(),
           amount: 1000,
        };

    //use rlp bytes
    //internally relies on rlp_append to perform the actual encoding(you can check bro !)
    let encoded_bytes = deposit.rlp_bytes();
        
    // Use rlp_append
    let mut stream = RlpStream::new();
    deposit.rlp_append(&mut stream);
    let encoded_append = stream.out();


    println!("Encoded using rlp_append: {:?}", encoded_append);
    println!("Encoded using rlp_bytes: {:?}",encoded_bytes);

    
    assert_eq!(encoded_bytes,encoded_append);



    //Decoding
    let rlp = Rlp::new(&encoded_bytes);
    let decoded_deposit = Deposit::decode(&rlp).unwrap();
    println!("Decoded deposit: {:?}", decoded_deposit);

    assert_eq!(deposit,decoded_deposit);

        
    }



}