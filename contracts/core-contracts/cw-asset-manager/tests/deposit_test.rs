mod setup;
use cosmwasm_std::{Addr, Uint128};
use cw_common::asset_manager_msg::ExecuteMsg;
use cw_multi_test::Executor;

use crate::setup::{
    call_set_xcall_host, execute_config_x_call, instantiate_contracts, set_default_connection,
    setup_context, TestContext,
};
use cw20::{Cw20Contract, Cw20ExecuteMsg};

//test helper
fn depsit_cw20_token(mut ctx: TestContext, msg: ExecuteMsg) -> TestContext {
    let relay = ctx.get_xcall_connection();
    ctx = set_default_connection(ctx, relay);
    call_set_xcall_host(&mut ctx);

    let resp = ctx
        .app
        .execute_contract(ctx.sender.clone(), ctx.get_assetmanager_app(), &msg, &[])
        .unwrap();

    println!("deposite execution resp: {:?}", resp.events);
    ctx
}

fn increase_allowance(mut ctx: TestContext, amount: Uint128) -> (TestContext, Uint128) {
    let relay = ctx.get_xcall_connection();
    let am_addr = ctx.get_assetmanager_app();

    let spok_addr = ctx.get_cw20token_app();
    let token = Cw20Contract(ctx.get_cw20token_app());

    ctx = set_default_connection(ctx, relay);
    call_set_xcall_host(&mut ctx);

    let allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
        spender: am_addr.to_string(),
        amount,
        expires: Some(cw_utils::Expiration::Never {}),
    };
    ctx.app
        .execute_contract(ctx.sender.clone(), spok_addr, &allowance_msg, &[])
        .unwrap();
    let resp = token
        .allowance(&ctx.app.wrap(), ctx.sender.clone(), am_addr)
        .unwrap();

    (ctx, resp.allowance)
}

//check for manaul test modification in only transfer submsg atomic execution inside contract
fn check_balance(ctx: &TestContext, token: &Addr, account: &Addr) -> Uint128 {
    let token_contract = Cw20Contract(token.clone());
    let app_query_wrapper = ctx.app.wrap();
    token_contract.balance(&app_query_wrapper, account).unwrap()
}

#[test]
#[should_panic]
//must panic for msg execution from asset managaer on xcall contract
//contract3 -----> contract0
/**
 * Expected Testing Contract's Addresses
 * asset_manager -----> contract3
 * spok_token -----> contract1
 * source_x_call -----> contract0
 */
fn test_deposit() {
    let mut context = setup_context();
    context = instantiate_contracts(context);
    let spok_addr = context.get_cw20token_app();
    let source_x_call = context.get_xcall_app();

    context = execute_config_x_call(context, source_x_call);

    let deposit_msg = ExecuteMsg::Deposit {
        token_address: spok_addr.to_string(),
        amount: Uint128::new(100),
        to: None,
        data: None,
    };

    let (ctx, allowance) = increase_allowance(context, Uint128::new(1000));
    assert_eq!(allowance, Uint128::new(1000));
    let ctx = depsit_cw20_token(ctx, deposit_msg);
    //balance will be updated after transfer on manual submsg execution check
    let _bl = check_balance(&ctx, &spok_addr, &ctx.sender);
}
