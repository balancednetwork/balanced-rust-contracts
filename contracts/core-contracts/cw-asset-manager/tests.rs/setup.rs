use std::collections::HashMap;

use cosmwasm_std::Addr;
use cw_multi_test::App;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum TestApps {
    XCall,
    AssetManager,
}

pub struct TestContext {
    pub app: App,
    pub contracts: HashMap<TestApps, Addr>,
    pub sender: Addr,
    pub admin: Option<String>,
    pub caller: Option<String>,
}


impl TestContext {
    pub fn set_xcall_app(&mut self, addr: Addr) -> Option<Addr> {
        self.contracts.insert(TestApps::XCall, addr)
    }

    pub fn set_AssetManager_app(&mut self, addr: Addr) -> Option<Addr> {
        self.contracts.insert(TestApps::AssetManager, addr)
    }

    pub fn get_xcall_app(&self) -> Addr {
        return self.contracts.get(&TestApps::XCall).unwrap().clone();
    }

    pub fn get_AssetManager_app(&self) -> Addr {
        return self.contracts.get(&TestApps::AssetManager).unwrap().clone();
    }
}


pub fn setup_context() -> TestContext {
    let router = App::default();
    let sender = Addr::unchecked("sender");

    TestContext {
        app: router,
        contracts: HashMap::new(),
        sender,
        admin: None,
        caller: None
    }
}

mod instantiate_test {
    use common::rlp::encode;
    use cosmwasm_std::{Addr, Empty};
    use cw_common::{xcall_msg::{self, XCallMsg}, asset_manager_msg::{self, InstantiateMsg, ExecuteMsg}, xcall_data_types::{Deposit, WithdrawRequest, DepositRevert, WithdrawTo}};
    use cw_multi_test::{ContractWrapper, Executor, Contract};
    //use x_call_mock::contract::{execute, instantiate, query};
    
    use super::*;

    fn init_x_call(mut ctx: TestContext) -> TestContext {
        let code: Box<dyn Contract<Empty>> = Box::new(ContractWrapper::new(execute, instantiate, query));
        let code_id = ctx.app.store_code(code);

        let addr = ctx.app.instantiate_contract(
                code_id,
                ctx.sender.clone(),
                &xcall_msg::InstatiateMsg{
                },
                &[],
                "XCall",
                None)
                .unwrap();
        ctx.set_xcall_app(addr);
        ctx
    }

    fn init_token(mut ctx: TestContext, x_call_address: String) -> TestContext {
        use asset_manager_msg::{InstantiateMsg};
        use cw-asset-manager::{execute, instantiate, query};

            let code: Box<dyn Contract<Empty>> = Box::new(ContractWrapper::new(execute, instantiate, query));
            let code_id = ctx.app.store_code(code);


            let addr = ctx.app.instantitate_contract(
                code_id,
                ctx.sender.clone(),
                &InstantiateMsg{
                    x_call: Addr::unchecked(x_call_address).to_string(),
                    xcall_hub_address: "btp://0x1.icon/archway1qvqas572t6fx7af203mzygn7lgw5ywjt4y6q8e".to_owned(),
                },
                &[],
                "nativeToken",
                None)
                .unwrap();
                
            ctx.set_AssetManager_app(addr);
            ctx

    }

    fn execute_setup(mut ctx: TestContext) -> TestContext {
        let resp = ctx.app 
            .execute_contract(
                ctx.sender.clone(),
                ctx.get_AssetManager_app(),
                &ExecuteMsg::ConfigureXcall {
                    source_xcall: Addr::unchecked(ctx.get_xcall_app()).into_string(),
                    destination_contract: "btp://0x38.bsc/archway1qvqas572t6fx7af203mzygn7lgw5ywjt4y6q8e".to_owned(),
                },
                &[],
            )
            .unwrap();
        
        ctx
    }

    fn handle_call_message(mut ctx: TestContext) -> TestContext {
        let call_data = xcall_data_types::Deposit {
            token_address: Addr::unchecked("token").into_string(),
            from: "btp://0x38.bsc/archway1qvqas572t6fx7af203mzygn7lgw5ywjt4y6q8e".to_owned(),
            to: "btp://0x1.icon/archway123fdth".to_string(),
            amount: 1000,
        };

        let data = encode(&call_data).to_vec();

        let _resp = ctx.app
            .execute_contract(
                ctx.sender.clone(),
                ctx.get_xcall_app(),
                &XCallMsg::TestHandleCallMessage {
                    from: "btp://0x38.bsc/archway1qvqas572t6fx7af203mzygn7lgw5ywjt4y6q8e".to_owned(),
                    data,
                    hub_token: ctx.get_AssetManager_app().into_string(),
                },
                &[],
            )
            .unwrap();

        let call_data = DepositRevert {
            caller: "btp://0x1.icon/".to_owned()+ctx.sender.as_str(),
            amount: 100,
        };

        let data = encode(&call_data).to_vec();

        let _resp = ctx.app
            .execute_contract(
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


    fn setup_contract(mut ctx: TestContext) -> TestContext {
        ctx = init_x_call(ctx);
        let x_call_address = ctx.get_xcall_app().into_string();
        ctx = init_token(ctx, x_call_address);
        ctx
    }


    fn setup_test() -> TestContext {
        let mut context: TestContext = setup_context();
        context = setup_contract(context);
        context = execute_setup(context);
        context = handle_call_message(context);

        context //return
    }
    
    #[test]
    fn contract_test() {
        setup_test();
    }
}