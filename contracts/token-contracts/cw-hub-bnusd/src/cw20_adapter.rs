use cosmwasm_schema::cw_serde;
use cosmwasm_std::SubMsg;
use cosmwasm_std::{coin, to_binary, Addr, BankMsg, Binary, CosmosMsg, WasmMsg};
use cosmwasm_std::{Coin, MessageInfo};
use cw20_adapter::msg::ExecuteMsg as Cw20AdapterMsg;
use cw_common::hub_token_msg::ExecuteMsg as TokenExecuteMsg;

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
        let message = Cw20AdapterMsg::RedeemAndTransfer {
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
            msg: to_binary(&Cw20AdapterMsg::Receive {
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
    pub fn burn_user_cw20_token(&self, amount: u128, owner: &Addr) -> CosmosMsg {
        let msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.token_contract.to_string(),
            msg: to_binary(&TokenExecuteMsg::BurnFrom {
                owner: owner.to_string(),
                amount: amount.into(),
            })
            .unwrap(),
            funds: vec![],
        });
        msg
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

    pub fn split_adapter_funds(&self, info: &MessageInfo) -> (Vec<Coin>, Vec<Coin>) {
        let (adapter, other) = info
            .funds
            .clone()
            .into_iter()
            .partition(|f| f.denom == self.denom());
        return (adapter, other);
    }
}
