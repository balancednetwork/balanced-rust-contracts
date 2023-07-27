use std::collections::HashMap;

use cw_multi_test::{App, AppResponse};

use cw_asset_manager::contract::{execute, instantiate, query, reply};
use cw_common::x_call_msg::XCallExecuteMsg;
use cw_multi_test::{Contract, ContractWrapper, Executor};
use cw_xcall_ibc_connection::{
    execute as execute_conn, instantiate as instantiate_conn, query as query_conn,
    reply as reply_conn,
};
use cw_xcall_multi::msg::InstantiateMsg as XCallInstantiateMsg;
use cw_xcall_multi::{
    execute as execute_xcall, instantiate as instantiate_xcall, query as query_xcall,
    reply as reply_xcall,
};

use cw20::{Cw20Coin, MinterResponse};
use cw20_base::contract::{execute as CwExecute, instantiate as CwInstantiate, query as CwQuery};

use cosmwasm_std::{Addr, Attribute, Empty, Event, Uint128};

use cw_common::{
    asset_manager_msg::{ExecuteMsg, InstantiateMsg},
    network_address::NetId,
};

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum TestApps {
    XCall,
    AssetManager,
    CW20Token,
    XcallConnection,
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

    pub fn set_assetmanager_app(&mut self, addr: Addr) -> Option<Addr> {
        self.contracts.insert(TestApps::AssetManager, addr)
    }

    pub fn set_xcall_connection(&mut self, addr: Addr) -> Option<Addr> {
        self.contracts.insert(TestApps::XcallConnection, addr)
    }

    pub fn set_cw20_token(&mut self, addr: Addr) -> Option<Addr> {
        self.contracts.insert(TestApps::CW20Token, addr)
    }

    pub fn get_xcall_app(&self) -> Addr {
        return self.contracts.get(&TestApps::XCall).unwrap().clone();
    }

    pub fn get_xcall_connection(&self) -> Addr {
        return self
            .contracts
            .get(&TestApps::XcallConnection)
            .unwrap()
            .clone();
    }

    pub fn get_assetmanager_app(&self) -> Addr {
        return self.contracts.get(&TestApps::AssetManager).unwrap().clone();
    }

    pub fn get_cw20token_app(&self) -> Addr {
        return self.contracts.get(&TestApps::CW20Token).unwrap().clone();
    }
}

//initialize test context at the initial test state
pub fn setup_context() -> TestContext {
    let router = App::default();
    let sender = Addr::unchecked("sender");
    TestContext {
        app: router,
        contracts: HashMap::new(),
        sender,
        admin: None,
        caller: None,
    }
}

pub fn x_call_contract_setup() -> Box<dyn Contract<Empty>> {
    Box::new(
        ContractWrapper::new(execute_xcall, instantiate_xcall, query_xcall).with_reply(reply_xcall),
    )
}

pub fn asset_manager_contract_setup() -> Box<dyn Contract<Empty>> {
    Box::new(ContractWrapper::new(execute, instantiate, query).with_reply(reply))
}

pub fn cw20_contract_setup() -> Box<dyn Contract<Empty>> {
    Box::new(ContractWrapper::new(CwExecute, CwInstantiate, CwQuery))
}

pub fn x_call_connection_setup() -> Box<dyn Contract<Empty>> {
    Box::new(
        ContractWrapper::new(execute_conn, instantiate_conn, query_conn).with_reply(reply_conn),
    )
}

//--------------------------------INITIALIZER FUNCTION HELPERS---------------------------------------------------------
pub fn init_x_call(mut ctx: TestContext) -> TestContext {
    let code: Box<dyn Contract<Empty>> = x_call_contract_setup();
    let code_id = ctx.app.store_code(code);

    let _addr = ctx
        .app
        .instantiate_contract(
            code_id,
            ctx.sender.clone(),
            &XCallInstantiateMsg {
                network_id: "0x02.archway".to_string(),
                denom: "xcalToken".to_string(),
            },
            &[],
            "XCall",
            None,
        )
        .unwrap();
    ctx.set_xcall_app(_addr);
    ctx
}

pub fn init_xcall_connection_contract(mut ctx: TestContext) -> TestContext {
    let connection_contract_code_id = ctx.app.store_code(x_call_connection_setup());
    let connection_contract_addr = ctx
        .app
        .instantiate_contract(
            connection_contract_code_id,
            ctx.sender.clone(),
            &cw_xcall_ibc_connection::msg::InstantiateMsg {
                ibc_host: Addr::unchecked("ibc-core"),
                denom: "uarch".to_string(),
                port_id: "mock".to_string(),
                xcall_address: ctx.get_xcall_app(),
            },
            &[],
            "IBCConnection",
            Some(ctx.sender.clone().to_string()),
        )
        .unwrap();
    ctx.set_xcall_connection(connection_contract_addr);
    ctx
}

pub fn init_cw20_token_contract(mut ctx: TestContext) -> TestContext {
    let code: Box<dyn Contract<Empty>> = cw20_contract_setup();
    let cw20_id = ctx.app.store_code(code);

    let msg = cw20_base::msg::InstantiateMsg {
        name: "Spokey".to_string(),
        symbol: "SPOK".to_string(),
        decimals: 18,
        initial_balances: vec![Cw20Coin {
            address: ctx.sender.to_string(),
            amount: Uint128::new(5000),
        }],
        mint: Some(MinterResponse {
            minter: ctx.sender.to_string(),
            cap: None,
        }),
        marketing: None,
    };
    let spok_address = ctx
        .app
        .instantiate_contract(cw20_id, ctx.sender.clone(), &msg, &[], "SPOKE", None)
        .unwrap();

    ctx.set_cw20_token(spok_address);
    ctx
}

pub fn init_asset_manager(mut ctx: TestContext) -> TestContext {
    let code: Box<dyn Contract<Empty>> = asset_manager_contract_setup();
    let code_id = ctx.app.store_code(code);

    let _addr = ctx
        .app
        .instantiate_contract(
            code_id,
            ctx.sender.clone(),
            &InstantiateMsg {},
            &[],
            "XCall",
            None,
        )
        .unwrap();
    ctx.set_assetmanager_app(_addr);
    ctx
}

pub fn instantiate_contracts(mut ctx: TestContext) -> TestContext {
    ctx = init_x_call(ctx);
    ctx = init_cw20_token_contract(ctx);
    ctx = init_xcall_connection_contract(ctx);
    ctx = init_asset_manager(ctx);
    ctx
}

//-------------------------execute function helpers--------------------------------------------
pub fn call_set_xcall_host(ctx: &mut TestContext) -> AppResponse {
    ctx.app
        .execute_contract(
            ctx.sender.clone(),
            ctx.get_xcall_connection(),
            &cw_common_ibc::xcall_connection_msg::ExecuteMsg::SetXCallHost {
                address: ctx.get_xcall_app().to_string(),
            },
            &[],
        )
        .unwrap()
}

pub fn execute_config_x_call(mut ctx: TestContext, x_call: Addr) -> TestContext {
    let _resp = ctx
        .app
        .execute_contract(
            ctx.sender.clone(),
            ctx.get_assetmanager_app(),
            &ExecuteMsg::ConfigureXcall {
                source_xcall: Addr::unchecked(x_call).into_string(),
                destination_asset_manager: "0x01.icon/cx7866543210fedcba9876543210fedcba987654df"
                    .to_owned(),
            },
            &[],
        )
        .unwrap();

    ctx
}

//--------------------------------------------------------------------------------
pub fn to_attribute_map(attrs: &Vec<Attribute>) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for attr in attrs {
        map.insert(attr.key.clone(), attr.value.clone());
    }
    map
}

pub fn get_event(res: &AppResponse, event: &str) -> Option<HashMap<String, String>> {
    let event = res
        .events
        .iter()
        .filter(|e| e.ty == event)
        .collect::<Vec<&Event>>();
    if !event.is_empty() {
        let map = to_attribute_map(&event[0].attributes);
        return Some(map);
    }
    None
}

pub fn set_default_connection(mut context: TestContext, address: Addr) -> TestContext {
    let _response = context
        .app
        .execute_contract(
            context.sender.clone(),
            context.get_xcall_app(),
            &XCallExecuteMsg::SetDefaultConnection {
                nid: NetId::from("0x01.icon".to_owned()).to_string(),
                address,
            },
            &[],
        )
        .unwrap();
    context
}
