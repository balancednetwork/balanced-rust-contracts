use std::collections::HashMap;
use cosmwasm_std::{Addr, Empty};
use cw_common::{xcall_msg::{self, XCallMsg}, asset_manager_msg::{self, InstantiateMsg, ExecuteMsg}, xcall_data_types::{Deposit, WithdrawRequest, DepositRevert, WithdrawTo}};
use cw_multi_test::{ContractWrapper, Executor, Contract};
use super::*;

mod instantiate_test {
    use common::rlp::encode;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use super::*;

    #[test]
    fn contract_test() {
        let mut context: TestContext = setup_test();
        // Add further assertions in future
    }

    fn setup_test() -> TestContext {
        let mut context: TestContext = setup_context();
        context = setup_contract(context);
        context = execute_setup(context);
        context = handle_call_message(context);
        context
    }

    fn setup_contract(mut ctx: TestContext) -> TestContext {
        ctx = init_x_call(ctx);
        let x_call_address = ctx.get_xcall_app().into_string();
        ctx = init_token(ctx, x_call_address);
        ctx
    }

    fn init_x_call(mut ctx: TestContext) -> TestContext {
        let code: Box<dyn Contract<Empty>> = Box::new(ContractWrapper::new(execute, instantiate, query));
        let code_id = ctx.app.store_code(code);

        let addr = ctx.app.instantiate_contract(
            code_id,
            ctx.sender.clone(),
            &xcall_msg::InstatiateMsg{},
            &[],
            "XCall",
            None,
        ).unwrap();
        ctx.set_xcall_app(addr);
        ctx
    }

    fn init_token(mut ctx: TestContext, x_call_address: String) -> TestContext {
        let code: Box<dyn Contract<Empty>> = Box::new(ContractWrapper::new(execute, instantiate, query));
        let code_id = ctx.app.store_code(code);

        let addr = ctx.app.instantiate_contract(
            code_id,
            ctx.sender.clone(),
            &asset_manager_msg::InstantiateMsg{
                x_call: Addr::unchecked(x_call_address).to_string(),
                xcall_hub_address: "btp://0x1.icon/archway1qvqas572t6fx7af203mzygn7lgw5ywjt4y6q8e".to_owned(),
            },
            &[],
            "nativeToken",
            None,
        ).unwrap();
        ctx.set_AssetManager_app(addr);
        ctx
    }

    fn execute_setup(mut ctx: TestContext) -> TestContext {
        let resp = ctx.app.execute_contract(
            ctx.sender.clone(),
            ctx.get_AssetManager_app(),
            &ExecuteMsg::ConfigureXcall {
                source_xcall: Addr::unchecked(ctx.get_xcall_app()).into_string(),
                destination_contract: "btp://0x38.bsc/archway1qvqas572t6fx7af203mzygn7lgw5ywjt4y6q8e".to_owned(),
            },
            &[],
        ).unwrap();
        ctx
    }

    fn handle_call_message(mut ctx: TestContext) -> TestContext {
        let call_data = Deposit {
            token_address: Addr::unchecked("spok").into_string(),
            from: "btp://0x38.bsc/archway1qvqas572t6fx7af203mzygn7lgw5ywjt4y6q8e".to_owned(),
            to: "btp://0x1.icon/archway123fdth".to_string(),
            amount: 1000,
        };

        let data = encode(&call_data).to_vec();

        let _resp = ctx.app.execute_contract(
            ctx.sender.clone(),
            ctx.get_xcall_app(),
            &XCallMsg::TestHandleCallMessage {
                from: "btp://0x38.bsc/archway1qvqas572t6fx7af203mzygn7lgw5ywjt4y6q8e".to_owned(),
                data,
                hub_token: ctx.get_AssetManager_app().into_string(),
            },
            &[],
        ).unwrap();

        let call_data = DepositRevert {
            caller: "btp://0x1.icon/".to_owned()+ctx.sender.as_str(),
            amount: 100,
        };

        let data = encode(&call_data).to_vec();

        let _resp = ctx.app.execute_contract(
            ctx.sender.clone(),
            ctx.get_xcall_app(),
            &XCallMsg::TestHandleCallMessage {
                from: "btp://0x1.icon/archway1qvqas572t6fx7af203mzygn7lgw5ywjt4y6q8e".to_owned(),
                data,
                hub_token: ctx.get_AssetManager_app().into_string(),
            },
            &[],
        ).unwrap();

        let call_data = WithdrawRequest {
            token_address: Addr::unchecked("spok").to_string(),
            account: "btp://0x38.bsc/archway1qvqas572t6fx7af203mzygn7lgw5ywjt4y6q8e".to_owned(),
            amount: 100,
        };

        let data = encode(&call_data).to_vec();

        let _resp = ctx.app.execute_contract(
            ctx.sender.clone(),
            ctx.get_xcall_app(),
            &XCallMsg::TestHandleCallMessage {
                from: "btp://0x1.icon/archway1qvqas572t6fx7af203mzygn7lgw5ywjt4y6q8e".to_owned(),
                data,
                hub_token: ctx.get_AssetManager_app().into_string(),
            },
            &[],
        ).unwrap();

        let call_data = WithdrawTo {
            token_address: Addr::unchecked("spok").to_string(),
            account: "btp://0x38.bsc/archway1qvqas572t6fx7af203mzygn7lgw5ywjt4y6q8e".to_owned(),
            amount: 100,
        };

        let data = encode(&call_data).to_vec();

        let _resp = ctx.app.execute_contract(
            ctx.sender.clone(),
            ctx.get_xcall_app(),
            &XCallMsg::TestHandleCallMessage {
                from: "btp://0x1.icon/archway1qvqas572t6fx7af203mzygn7lgw5ywjt4y6q8e".to_owned(),
                data,
                hub_token: ctx.get_AssetManager_app().into_string(),
            },
            &[],
        ).unwrap();

        ctx
    }
}