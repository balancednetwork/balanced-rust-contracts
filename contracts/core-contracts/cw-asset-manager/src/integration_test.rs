use cosmwasm_std::{Addr, Empty, Uint128};
use cw20::Cw20Coin;
use cw_multi_test::{App, Contract, ContractWrapper, Executor};

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

fn mock_app() -> App {
    App::default()
}

/* //////////////////////////////////////////////////
            CHECK cw20 instantiate
/////////////////////////////////////////////////// */

#[test]

fn test_cw20_instantiate() {
    let mut app = mock_app();
    let owner = Addr::unchecked("owner");
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
    let resp = app.instantiate_contract(cw20_id, owner.clone(), &msg, &[], "SPOKE", None);
    assert!(resp.is_ok());
}

/* //////////////////////////////////////////////////
            check asset manager instantiation
/////////////////////////////////////////////////// */
#[test]

fn test_asset_manager_instantiation() {
    let mut app = mock_app();
    let owner = Addr::unchecked("owner");
    let asset_id = app.store_code(contract_assetmanager());
    let msg = InstantiateMsg {};

    let asst_addr =
        app.instantiate_contract(asset_id, owner.clone(), &msg, &[], "asset_manager", None);
    assert!(asst_addr.is_ok());
}

/* //////////////////////////////////////////////////
            testing the cw20 deposit
/////////////////////////////////////////////////// */
#[test]

fn test_deposit() {
    let mut app = mock_app();

    // assigned  owner
    let owner = Addr::unchecked("owner");
    let cw20_id = app.store_code(contract_cw20());
    let init_msg = cw20_base::msg::InstantiateMsg {
        name: "Spokey".to_string(),
        symbol: "SPOK".to_string(),
        decimals: 18,
        initial_balances: vec![Cw20Coin {
            address: owner.clone().to_string(),
            amount: Uint128::new(5000),
        }],
        mint: None,
        marketing: None,
    };
    let resp = app
        .instantiate_contract(cw20_id, owner.clone(), &init_msg, &[], "SPOKE", None)
        .unwrap();

    let asset_id = app.store_code(contract_assetmanager());

    let msg = InstantiateMsg {};

    let asset_addr = app
        .instantiate_contract(asset_id, owner.clone(), &msg, &[], "asset_manager", None)
        .unwrap();
    let msg = ExecuteMsg::Deposit {
        token_address: resp.clone().to_string(),
        amount: Uint128::new(1500128),
        to: Some(String::from(
            "0x01.icon/cx9876543210fedcba9876543210fedcba98765432",
        )),
        data: None,
    };

    let _exec_resp = app.execute_contract(owner.clone(), asset_addr.clone(), &msg, &[]);

    let msg = cw20::Cw20ExecuteMsg::Transfer {
        recipient: String::from("0x01.icon/cx9876543210fedcba9876543210fedcba98765432"),
        amount: Uint128::new(100),
    };

    let exec_deposit = app.execute_contract(owner.clone(), resp.clone(), &msg, &[]);

    //     for attribute in &exec_deposit.events[0].attributes {
    //         match attribute {
    //             Attribute { key, value } => match key.as_str() {
    //                 "Token" => assert_eq!(value, "Spokey"),
    //                 "To" => println!("value: {:?}",value),
    //                 "Amount" => assert_eq!(value, "1500"),
    //                 _ => panic!("Unexpected attribute key"),
    //             },
    //     }
    // }

    assert!(exec_deposit.is_ok());
}

#[test]

fn test_deposit_cw20_tokens_with_zero_amount() {
    let mut app = mock_app();

    // assigned  owner
    let owner = Addr::unchecked("owner");
    let cw20_id = app.store_code(contract_cw20());
    let init_msg = cw20_base::msg::InstantiateMsg {
        name: "Spokey".to_string(),
        symbol: "SPOK".to_string(),
        decimals: 18,
        initial_balances: vec![Cw20Coin {
            address: owner.clone().to_string(),
            amount: Uint128::zero(),
        }],
        mint: None,
        marketing: None,
    };
    let resp = app
        .instantiate_contract(cw20_id, owner.clone(), &init_msg, &[], "SPOKE", None)
        .unwrap();

    let asset_id = app.store_code(contract_assetmanager());

    let msg = InstantiateMsg {};

    let asset_addr = app
        .instantiate_contract(asset_id, owner.clone(), &msg, &[], "asset_manager", None)
        .unwrap();
    let msg = ExecuteMsg::Deposit {
        token_address: resp.clone().to_string(),
        amount: Uint128::zero(),
        to: Some(String::from(
            "0x01.icon/cx9876543210fedcba9876543210fedcba98765432",
        )),
        data: None,
    };

    let _exec_resp = app.execute_contract(owner.clone(), asset_addr.clone(), &msg, &[]);

    let msg = cw20::Cw20ExecuteMsg::TransferFrom {
        owner: owner.clone().to_string(),
        recipient: String::from("0x01.icon/cx9876543210fedcba9876543210fedcba98765432"),
        amount: Uint128::zero(),
    };

    let exec_deposit = app.execute_contract(owner.clone(), resp.clone(), &msg, &[]);

    assert!(exec_deposit.is_err());
}
