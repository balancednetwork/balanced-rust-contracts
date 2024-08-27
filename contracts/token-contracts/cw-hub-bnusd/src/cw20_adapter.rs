use cosmwasm_schema::cw_serde;
use cosmwasm_std::SubMsg;
use cosmwasm_std::{coin, to_binary, Addr, BankMsg, Binary, CosmosMsg, Deps, Env, WasmMsg};
use cosmwasm_std::{MessageInfo, Uint128};
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

#[cw_serde]
pub struct CW20Adapter {
    token_contract: Addr,
    adapter_contract: Addr,
}

impl CW20Adapter {
    pub fn new(token_contract: Addr, adapter_contract: Addr) -> Self {
        return Self {
            token_contract,
            adapter_contract,
        };
    }
    // convert specified TF tokens to our token and transfer to receiver
    pub fn redeem(&self, amount: u128, receiver: &Addr) -> SubMsg {
        let fund = coin(amount, self.denom());
        let message = RegistryExecuteMsg::RedeemAndTransfer {
            recipient: Some(receiver.to_string()),
        };

        let msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.adapter_contract.to_string(),
            msg: to_binary(&message).unwrap(),
            funds: vec![fund],
        });
        let submessage = SubMsg {
            id: 2,
            msg,
            gas_limit: None,
            reply_on: cosmwasm_std::ReplyOn::Never,
        };
        submessage
    }
    // mint equivalent TF tokens for the receiver
    pub fn receive(&self, receiver: &Addr, amount: u128) -> SubMsg {
        let msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.adapter_contract.to_string(),
            msg: to_binary(&RegistryExecuteMsg::Receive {
                sender: receiver.to_string(),
                amount: amount.into(),
                msg: Binary::default(),
            })
            .unwrap(),
            funds: vec![],
        });
        let submessage = SubMsg {
            id: 1,
            msg,
            gas_limit: None,
            reply_on: cosmwasm_std::ReplyOn::Never,
        };
        submessage
    }
    // transfer tf tokens from this contract to user.
    pub fn transfer(&self, recepient: &Addr, amount: u128) -> CosmosMsg {
        return CosmosMsg::Bank(BankMsg::Send {
            to_address: recepient.to_string(),
            amount: vec![coin(amount, self.denom())],
        });
    }
    // burn users cw20 tokens after redeem
    pub fn burn_user_cw20_token(&self, amount: u128) -> CosmosMsg {
        return CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.token_contract.to_string(),
            msg: to_binary(&TokenExecuteMsg::Burn {
                amount: amount.into(),
            })
            .unwrap(),
            funds: vec![],
        });
    }

    pub fn denom(&self) -> String {
        format!("factory/{}/{}", self.adapter_contract, self.token_contract)
    }

    pub fn adapter_contract(&self) -> &Addr {
        &self.adapter_contract
    }

    pub fn get_adapter_fund(&self, info: &MessageInfo) -> u128 {
        info.funds
            .iter()
            .filter_map(|f| {
                if f.denom == self.denom() {
                    return Some(f.amount.u128());
                }
                None
            })
            .sum()
    }
}
