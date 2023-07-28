mod setup;

use cosmwasm_std::Addr;
use cw_common::{
    network_address::{NetId, NetworkAddress},
    x_call_msg::XCallExecuteMsg,
    xcall_data_types::WithdrawTo,
};
use cw_multi_test::Executor;
use rlp::{Encodable, RlpStream};
use setup::{
    execute_config_x_call, get_event, instantiate_contracts, set_default_connection, setup_context,
    TestContext,
};

fn execute_handle_msg_on_asset_manager_from_relayer(mut ctx: TestContext) -> TestContext {
    let token_addr = ctx.get_cw20token_app().to_string();
    let relay = Addr::unchecked("relayer");
    let asset_manager = ctx.get_assetmanager_app();

    // ----------------------------   execution flow from RELAYER------>  XCALL --------------------------------------------

    //pretend relayer for the connection such that relay can call ExecuteCall msg on xcall
    ctx = set_default_connection(ctx, relay.clone());

    let call_data = WithdrawTo {
        token_address: token_addr,
        user_address: "sender".to_string(),
        amount: 100,
    };

    let data = call_data.rlp_bytes().to_vec();

    //random seq_no. for test purpose
    let sn: u64 = 100;
    let msg_typ: u64 = 1;

    //destination asset manager
    let from = NetworkAddress("0x01.icon/cx7866543210fedcba9876543210fedcba987654df".to_owned());

    //construct encoded CallServiceMessageRequest
    let mut stream = RlpStream::new();
    stream.begin_list(6);
    stream.append(&from.to_string());
    stream.append(&asset_manager.to_string());
    stream.append(&sn);
    stream.append(&false);
    stream.append(&data);
    stream.begin_list(0);

    let encoded_data = stream.out().to_vec();

    let mut stream = RlpStream::new();
    stream.begin_list(2);
    stream.append(&msg_typ);
    stream.append(&encoded_data);

    //construct encoded CallServiceMessage
    let msg_data = stream.out().to_vec();
    let response = ctx.app.execute_contract(
        relay.clone(),
        ctx.get_xcall_app(),
        &XCallExecuteMsg::HandleMessage {
            from: NetId::from("0x01.icon".to_owned()).to_string(),
            sn: None,
            msg: msg_data,
        },
        &[],
    );

    let event = get_event(&response.unwrap(), "wasm-CallMessage").unwrap();
    // *`request_id`: `request_id` is a unique identifier for a specific request made by a user. It is
    // used to retrieve the details of the request from the contract's storage and execute the
    // corresponding action.
    let value = event.get("reqId").unwrap();
    let req_id = value.parse::<u128>().unwrap();

    // ----------------------------        execution flow from XCALL------> ASSET MANAGER       --------------------------------------------
    let response = ctx
        .app
        .execute_contract(
            relay,
            ctx.get_xcall_app(),
            &XCallExecuteMsg::ExecuteCall {
                request_id: req_id,
                data,
            },
            &[],
        )
        .unwrap();

    println!("withdraw Resp: {:?}", response);

    ctx
}

#[test]
fn handle_call_message_test_for_withdraw_to() {
    let mut context = setup_context();
    context = instantiate_contracts(context);
    let source_x_call = context.get_xcall_app();
    context = execute_config_x_call(context, source_x_call);
    execute_handle_msg_on_asset_manager_from_relayer(context);
}
