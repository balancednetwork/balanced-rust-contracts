#![cfg(test)]

use cosmwasm_std::{to_binary, Addr, Empty, Uint128};
use cw20::{Cw20Coin, Cw20Contract, Cw20ExecuteMsg};
use cw_multi_test::{App, Contract, ContractWrapper, Executor};

use super::contract_helper::AssetManagerContract;
use cw_common::asset_manager_msg::*;

const OWNER: &str = "owner";

pub fn contract_assetmanager() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

pub fn contract_cw20() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    );
    Box::new(contract)
}

pub fn contract_xcall() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        x_call_mock::contract::execute,
        x_call_mock::contract::instantiate,
        x_call_mock::contract::query,
    );
    Box::new(contract)
}

fn setup_cw20_contract(app: &mut App, owner: Addr) -> Addr {
    let cw20_id = app.store_code(contract_cw20());
    let msg = cw20_base::msg::InstantiateMsg {
        name: "Spokey".to_string(),
        symbol: "SPOK".to_string(),
        decimals: 18,
        initial_balances: vec![Cw20Coin {
            address: OWNER.to_string(),
            amount: Uint128::new(5000),
        }],
        mint: None,
        marketing: None,
    };
    app.instantiate_contract(cw20_id, owner.clone(), &msg, &[], "SPOKE", None)
        .unwrap()
}

fn setup_asset_manager_contract(app: &mut App, owner: Addr) -> Addr {
    let asset_manager_id = app.store_code(contract_assetmanager());
    app.instantiate_contract(
        asset_manager_id,
        owner.clone(),
        &InstantiateMsg {},
        &[],
        "ASSET",
        None,
    )
    .unwrap()
}

#[test]
fn cw20_token_deposit() {
    let mut app = App::default();
    let owner = Addr::unchecked("owner");

    //contract instances
    let spok = Cw20Contract(setup_cw20_contract(&mut app, owner.to_owned()));
    let asset_manager = AssetManagerContract(setup_asset_manager_contract(&mut app, owner.clone()));
    let am_address = asset_manager.addr();

    //check initial spok balance of the owner: expected(5000)
    let owner_balance = spok.balance(&app.wrap(), owner.to_owned()).unwrap();
    assert_eq!(owner_balance, Uint128::new(5000));

    //provide allowance for asset_manager from owner
    let allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
        spender: am_address.into(),
        amount: Uint128::new(1000),
        expires: None,
    };
    app.execute_contract(owner.clone(), spok.addr(), &allowance_msg, &[])
        .unwrap();

    let am: String = am_address.into();
    //check allowance: (expected :1000 spok)
    let allowance_resp = spok.allowance(&app.wrap(), owner.clone(), am).unwrap();
    println!("allowance resp: {:?}", allowance_resp);

    let resp = asset_manager.deposit(
        &owner,
        &mut app,
        &(spok.addr()).to_string(),
        &Uint128::new(500),
        None,
        None,
    );
    println!("deposit resp: {:?}", resp);
}
