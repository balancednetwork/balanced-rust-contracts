use cosmwasm_std::Querier;
#[cfg(test)]

use cosmwasm_std::{to_binary, Addr, Empty, Uint128};
use cw20::{Cw20Coin, Cw20Contract, Cw20ExecuteMsg};
use cw_multi_test::{App, Contract, ContractWrapper, Executor};

use super::contract_helper::AssetManagerContract;
use cw_common::asset_manager_msg::*;

const OWNER: &str = "owner";

fn mock_app() -> App {
    App::default()
}


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


// Test must throws error cause owner can deposit 0 amount , validation must added
#[test]
fn cw20_token_deposit_with_zero() {
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
        &Uint128::zero(),
        None,
        None,
    );
    println!("deposit resp: {:?}", resp);
}  

#[test]
// #[should_panic(expected = "Zero amount deposit not allowed")]
fn cw20_token_deposit_with_random_address() {
    let mut app = mock_app();
    let user = Addr::unchecked("user");
    let owner = Addr::unchecked("owner");

    // contract instances from owner side

    let spok = Cw20Contract(setup_cw20_contract(&mut app, owner.to_owned()));
    let asset_manager = AssetManagerContract(setup_asset_manager_contract(&mut app, owner.to_owned()));

    // Inital deposited balance of owner while instantiating is 5000
    let  owner_balance = spok.balance(&app.wrap(), owner.to_owned()).unwrap();
    assert_eq!(owner_balance, Uint128::new(5000));

    // let check user balance expected zero 

    let user_balance = spok.balance(&app.wrap(), user.to_owned()).unwrap();
    assert_eq!(user_balance, Uint128::zero());

    let resp = asset_manager.deposit(
        &user,
        &mut app,
        &(spok.addr()).to_string(),
        &Uint128::zero(),
        None,
        None,
    );

    assert!(resp.is_err());

}

#[test]
//#[should_panic(expected = "Insufficient token allowance: CW20")]
// should throws insufficient token allowance if is less than the 
fn cw20_token_deposit_with_less_allowance() {
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
        &Uint128::zero(),
        None,
        None,
    );
    println!("deposit resp: {:?}", resp);
}

#[test]

fn configure_xcall() {
    let mut app = App::default();
    let owner = Addr::unchecked("owner");

    let source_xcall = Addr::unchecked("source_xcall");
    let destination_asset_contract = Addr::unchecked("destination_asset_contract");


    let asset_manager = AssetManagerContract(setup_asset_manager_contract(&mut app, owner.clone()));
    let am_address = asset_manager.addr();

    let query_msg = XCALLQuery::GetNetworkAddress {};


    let resp = asset_manager.configure_xcall(&owner, &mut app, source_xcall.to_string(), destination_asset_contract.to_string());

    println!("deposit resp: {:?}", resp);
    assert!(resp.is_ok());

}