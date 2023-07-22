use cosmwasm_std::{Addr, Uint128};
use cw_multi_test::Executor;
use setup::{TestContext, setup_context, instantiate_contracts, execute_setup};
use cw20::{BalanceResponse, AllowanceResponse};

mod setup;

#[test]
fn cw20_flow_test() {
    let mut context: TestContext = setup_context();
    context = instantiate_contracts(context);
    context = execute_setup(context);

    let alice = Addr::unchecked("alice".to_owned());
    let bob = Addr::unchecked("bob".to_owned());
    let carol = Addr::unchecked("carol".to_owned());
    let amount: u128 = 1000;

    //mint 1000 tokens to each account, and minting access is given to only xcall app
    let resp = context
        .app
        .execute_contract(
            context.get_hubtoken_app(),
            context.get_xcall_app(),
            &cw_common::hub_token_msg::ExecuteMsg::Mint {
                recipient: alice.to_string(),
                amount: Uint128::from(amount),
            },
            &[],
        );

        assert_eq!(resp.is_err(), true);//cannot mint tokens from hubtoken app

        //use loop to mint tokens from xcall app to alice, bob and carol
    vec![alice.to_string(), bob.to_string(), carol.to_string()]
    .iter()
    .for_each(|recipient| {
        let _resp = context
            .app
            .execute_contract(
                context.get_xcall_app().clone(),
                context.get_hubtoken_app(),
                &cw_common::hub_token_msg::ExecuteMsg::Mint {
                    recipient: recipient.clone(),
                    amount: Uint128::from(amount),
                },
                &[],
            )
            .unwrap();
    });
    
    //check balance of each account, and assert this to be 1000
    vec![
        (alice.to_string(), amount),
        (bob.to_string(), amount),
        (carol.to_string(), amount),
    ]
    .iter()
    .for_each(|(account, balance)| {
        let balance_response: BalanceResponse = context
            .app
            .wrap()
            .query_wasm_smart(
                context.get_hubtoken_app(),
                &cw_common::hub_token_msg::QueryMsg::Balance {
                    address: account.to_string(),
                },
            )
            .unwrap();
        println!("balance: {:?}", balance_response.balance.u128());
        assert_eq!(balance_response.balance.u128(), *balance);
    });

    //transfer 100 tokens from alice to bob and check again balance
    let transfer_amount:u128 = 100;
    let _resp = context
        .app
        .execute_contract(
            alice.clone(),
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::ExecuteMsg::Transfer {
                recipient: bob.to_string(),
                amount: Uint128::from(transfer_amount),
            },
            &[],
        )
        .unwrap();

    let balance_response: BalanceResponse = context.app.wrap().query_wasm_smart(
        context.get_hubtoken_app(),
        &cw_common::hub_token_msg::QueryMsg::Balance {
            address: alice.to_string(),
        },
    ).unwrap();
    assert_eq!(balance_response.balance.u128(), amount - transfer_amount);

    let balance_response: BalanceResponse = context.app.wrap().query_wasm_smart(
        context.get_hubtoken_app(),
        &cw_common::hub_token_msg::QueryMsg::Balance {
            address: bob.to_string(),
        },
    ).unwrap();
    assert_eq!(balance_response.balance.u128(), amount + transfer_amount);

    //transfer 100 tokens from bob to carol and check again balance
    let _resp = context
        .app
        .execute_contract(
            bob.clone(),
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::ExecuteMsg::Transfer {
                recipient: carol.to_string(),
                amount: Uint128::from(transfer_amount),
            },
            &[],
        )
        .unwrap();

    let balance_response: BalanceResponse = context.app.wrap().query_wasm_smart(
        context.get_hubtoken_app(),
        &cw_common::hub_token_msg::QueryMsg::Balance {
            address: bob.to_string(),
        },
    ).unwrap();
    assert_eq!(balance_response.balance.u128(), amount - transfer_amount + transfer_amount);

    let balance_response: BalanceResponse = context.app.wrap().query_wasm_smart(
        context.get_hubtoken_app(),
        &cw_common::hub_token_msg::QueryMsg::Balance {
            address: carol.to_string(),
        },
    ).unwrap();
    assert_eq!(balance_response.balance.u128(), amount + transfer_amount);

    //check self transfer, which should fail

    let _resp = context
        .app
        .execute_contract(
            alice.clone(),
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::ExecuteMsg::Transfer {
                recipient: alice.to_string(),
                amount: Uint128::from(transfer_amount),
            },
            &[],
        );
    assert_eq!(_resp.is_err(), true);

    //set allowance of 100 tokens from alice to bob and and transfer 50 tokens of alice from bob to carol

    let _resp = context
        .app
        .execute_contract(
            alice.clone(),
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::ExecuteMsg::IncreaseAllowance {
                spender: bob.to_string(),
                amount: Uint128::from(transfer_amount),
                expires: None,
            },
            &[],
        )
        .unwrap();

    let balance_response: BalanceResponse = context.app.wrap().query_wasm_smart(
        context.get_hubtoken_app(),
        &cw_common::hub_token_msg::QueryMsg::Balance {
            address: alice.to_string(),
        },
    ).unwrap();
    assert_eq!(balance_response.balance.u128(), amount - transfer_amount);

    let allowance_response:AllowanceResponse = context.app.wrap().query_wasm_smart(
        context.get_hubtoken_app(),
        &cw_common::hub_token_msg::QueryMsg::Allowance {
            owner: alice.to_string(),
            spender: bob.to_string(),
        },
    ).unwrap();
    assert_eq!(allowance_response.allowance.u128(), transfer_amount);
    let _resp = context
        .app
        .execute_contract(
            bob.clone(),
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::ExecuteMsg::TransferFrom {
                owner: alice.to_string(),
                recipient: carol.to_string(),
                amount: Uint128::from(transfer_amount/2),
            },
            &[],
        )
        .unwrap();

    let balance_response: BalanceResponse = context.app.wrap().query_wasm_smart(
        context.get_hubtoken_app(),
        &cw_common::hub_token_msg::QueryMsg::Balance {
            address: alice.to_string(),
        },
    ).unwrap();
    assert_eq!(balance_response.balance.u128(), amount - transfer_amount - transfer_amount/2);
        //get allowance of alice to bob and assert it to be 50
    let allowance_response: AllowanceResponse = context.app.wrap().query_wasm_smart(
        context.get_hubtoken_app(),
        &cw_common::hub_token_msg::QueryMsg::Allowance {
            owner: alice.to_string(),
            spender: bob.to_string(),
        },
    ).unwrap();
    assert_eq!(allowance_response.allowance.u128(), transfer_amount - transfer_amount/2);

    //increase, decrease and check allowance

    let _resp = context
        .app
        .execute_contract(
            alice.clone(),
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::ExecuteMsg::IncreaseAllowance {
                spender: bob.to_string(),
                amount: Uint128::from(transfer_amount),
                expires: None,
            },
            &[],
        )
        .unwrap();

    let allowance_response: AllowanceResponse = context.app.wrap().query_wasm_smart(
        context.get_hubtoken_app(),
        &cw_common::hub_token_msg::QueryMsg::Allowance {
            owner: alice.to_string(),
            spender: bob.to_string(),
        },
    ).unwrap();
    assert_eq!(allowance_response.allowance.u128(), transfer_amount+transfer_amount/2);

    let _resp = context
        .app
        .execute_contract(
            alice.clone(),
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::ExecuteMsg::DecreaseAllowance {
                spender: bob.to_string(),
                amount: Uint128::from(transfer_amount),
                expires: None,
            },
            &[],
        )
        .unwrap();


    let allowance_response: AllowanceResponse = context.app.wrap().query_wasm_smart(
        context.get_hubtoken_app(),
        &cw_common::hub_token_msg::QueryMsg::Allowance {
            owner: alice.to_string(),
            spender: bob.to_string(),
        },
    ).unwrap();
    assert_eq!(allowance_response.allowance.u128(), transfer_amount/2);

    //burn 100 tokens from alice and check balance
    let _resp = context
        .app
        .execute_contract(
            alice.clone(),
            context.get_hubtoken_app(),
            &cw_common::hub_token_msg::ExecuteMsg::Burn {
                amount: Uint128::from(transfer_amount),
            },
            &[],
        )
        .unwrap();

    let balance_response: BalanceResponse = context.app.wrap().query_wasm_smart(
        context.get_hubtoken_app(),
        &cw_common::hub_token_msg::QueryMsg::Balance {
            address: alice.to_string(),
        },
    ).unwrap();
    assert_eq!(balance_response.balance.u128(), amount - transfer_amount - transfer_amount/2 - transfer_amount);
}