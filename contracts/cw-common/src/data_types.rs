use common::rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

use crate::network_address::NetworkAddress;

#[cw_serde]
pub struct CrossTransfer {
    pub method: String,
    pub from: NetworkAddress,
    pub to: NetworkAddress,
    pub value: u128,
    pub data: Vec<u8>,
}

#[cw_serde]
pub struct CrossTransferRevert {
    pub method: String,
    pub from: Addr,
    pub value: u128,
}

impl Encodable for CrossTransfer {
    fn rlp_append(&self, stream: &mut RlpStream) {
        stream
            .begin_list(5)
            .append(&self.method)
            .append(&self.from.to_string())
            .append(&self.to.to_string())
            .append(&self.value)
            .append(&self.data);
    }
}

impl Decodable for CrossTransfer {
    fn decode(rlp: &Rlp<'_>) -> Result<CrossTransfer, DecoderError> {
        Ok(Self {
            method: rlp.val_at(0)?,
            from: NetworkAddress(rlp.val_at(1)?),
            to: NetworkAddress(rlp.val_at(2)?),
            value: rlp.val_at(3)?,
            data: rlp.val_at(4)?,
        })
    }
}

impl Encodable for CrossTransferRevert {
    fn rlp_append(&self, stream: &mut RlpStream) {
        stream
            .begin_list(3)
            .append(&self.method)
            .append(&self.from.to_string())
            .append(&self.value);
    }
}

impl Decodable for CrossTransferRevert {
    fn decode(rlp: &Rlp<'_>) -> Result<CrossTransferRevert, DecoderError> {
        let from: String = rlp.val_at(1)?;
        Ok(Self {
            method: rlp.val_at(0)?,
            from: Addr::unchecked(from),
            value: rlp.val_at(2)?,
        })
    }
}
