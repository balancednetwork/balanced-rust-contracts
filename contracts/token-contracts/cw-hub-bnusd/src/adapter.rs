use super::state::REGISTRY_ADDRESS;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;
use cosmwasm_std::{coin, to_binary, Addr, BankMsg, Binary, CosmosMsg, Deps, Env, WasmMsg};
use cw_common::hub_token_msg::ExecuteMsg as TokenExecuteMsg;

#[cw_serde]
pub enum RegistryExecuteMsg {
    /// Registers a new CW-20 contract that will be handled by the adapter
    RegisterCw20Contract { addr: Addr },
    ///  Impl of Receiver CW-20 interface. Should be called by CW-20 contract only!! (never directly). Msg is ignored
    Receive {
        sender: String,
        amount: Uint128,
        msg: Binary,
    },
    /// Called to redeem TF tokens. Will send CW-20 tokens to "recipient" address (or sender if not provided). Will use transfer method
    RedeemAndTransfer { recipient: Option<String> },
    /// Called to redeem TF tokens. Will call Send method of CW:20 to send CW-20 tokens to "recipient" address. Submessage will be passed to send method (can be empty)
    RedeemAndSend { recipient: String, submsg: Binary },
    /// Updates stored metadata
    UpdateMetadata { addr: Addr },
}

pub struct Adapter {
    token_contract: Addr,
    registry_contract: Addr,
}

impl Adapter {
    pub fn new(env: &Env, deps: Deps) -> Self {
        return Self {
            token_contract: env.contract.address.clone(),
            registry_contract: REGISTRY_ADDRESS.load(deps.storage).unwrap(),
        };
    }

    pub fn redeem(&self, amount: u128) -> CosmosMsg {
        let fund = coin(amount, self.denom());
        let message = RegistryExecuteMsg::RedeemAndTransfer {
            recipient: Some(self.token_contract.to_string()),
        };

        return CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.registry_contract.to_string(),
            msg: to_binary(&message).unwrap(),
            funds: vec![fund],
        });
    }

    pub fn deposit(&self, amount: u128) -> CosmosMsg {
        return CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.token_contract.to_string(),
            msg: to_binary(&TokenExecuteMsg::Send {
                contract: self.registry_contract.to_string(),
                amount: amount.into(),
                msg: Binary::default(),
            })
            .unwrap(),
            funds: vec![],
        });
    }

    pub fn transfer(&self, recepient: &Addr, amount: u128) -> CosmosMsg {
        return CosmosMsg::Bank(BankMsg::Send {
            to_address: recepient.to_string(),
            amount: vec![coin(amount, self.denom())],
        });
    }

    pub fn denom(&self) -> String {
        format!("factory/{}/{}", self.registry_contract, self.token_contract)
    }
}
