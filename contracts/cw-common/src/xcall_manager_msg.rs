use std::vec;

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use cw_ibc_rlp_lib::rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

#[cw_serde]
pub struct InstantiateMsg {
    pub xcall: Addr,
    pub icon_governance: String,
    pub protocols: ProtocolConfig,
}

#[cw_serde]
pub enum ExecuteMsg {
    ProposeChange {
        protocol: String,
    },
    RemoveProposal {},
    ChangeProposer {
        proposer: Addr,
    },
    HandleCallMessage {
        from: String,
        data: Vec<u8>,
        protocols: Option<Vec<String>>,
    },
}

#[cw_serde]
pub struct ProtocolConfig {
    pub sources: Vec<String>,
    pub destinations: Vec<String>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(bool)]
    VerifyProtocols { protocols: Vec<String> },
    #[returns(ProtocolConfig)]
    GetProtocols {},
}

#[cw_serde]
pub struct MigrateMsg {}

pub const CONFIGURE_PROTOCOLS: &str = "ConfigureProtocols";
#[cw_serde]
pub struct ConfigureProtocols {
    pub sources: Vec<String>,
    pub destinations: Vec<String>,
}

impl Encodable for ConfigureProtocols {
    fn rlp_append(&self, stream: &mut RlpStream) {
        stream.begin_list(3);
        stream.append(&CONFIGURE_PROTOCOLS.to_string());
        stream.begin_list(self.sources.len());
        for protocol in self.sources.iter() {
            stream.append(protocol);
        }

        stream.begin_list(self.destinations.len());
        for protocol in self.destinations.iter() {
            stream.append(protocol);
        }
    }
}

impl Decodable for ConfigureProtocols {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        let rlp_sources = rlp.at(1)?;
        let rlp_destinations = rlp.at(2)?;
        Ok(Self {
            sources: rlp_sources.as_list()?,
            destinations: rlp_destinations.as_list()?,
        })
    }
}

pub const EXECUTE: &str = "Execute";
#[cw_serde]
pub struct Execute {
    pub contract_addr: String,
    pub message: String, // Base64 encoded binary
}

impl Encodable for Execute {
    fn rlp_append(&self, stream: &mut RlpStream) {
        stream.begin_list(3);
        stream.append(&EXECUTE.to_string());
        stream.append(&self.contract_addr.to_string());
        stream.append(&self.message.to_string());
    }
}

impl Decodable for Execute {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        Ok(Self {
            contract_addr: rlp.val_at(1)?,
            message: rlp.val_at(2)?,
        })
    }
}

pub const MIGRATE: &str = "Migrate";

#[cw_serde]
pub struct Migrate {
    pub contract_addr: String,
    pub code_id: u64,
    pub message: String,
}

impl Encodable for Migrate {
    fn rlp_append(&self, stream: &mut RlpStream) {
        stream.begin_list(3);
        stream.append(&MIGRATE.to_string());
        stream.append(&self.contract_addr.to_string());
        stream.append(&self.code_id);
        stream.append(&self.message.to_string());
    }
}

impl Decodable for Migrate {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        Ok(Self {
            contract_addr: rlp.val_at(1)?,
            code_id: rlp.val_at(2)?,
            message: rlp.val_at(3)?,
        })
    }
}

pub const UPDATE_ADMIN: &str = "UpdateAdmin";
#[cw_serde]
pub struct UpdateAdmin {
    pub contract_addr: String,
    pub admin: String,
}

impl Encodable for UpdateAdmin {
    fn rlp_append(&self, stream: &mut RlpStream) {
        stream.begin_list(3);
        stream.append(&UPDATE_ADMIN.to_string());
        stream.append(&self.contract_addr.to_string());
        stream.append(&self.admin);
    }
}

impl Decodable for UpdateAdmin {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        Ok(Self {
            contract_addr: rlp.val_at(1)?,
            admin: rlp.val_at(2)?,
        })
    }
}
