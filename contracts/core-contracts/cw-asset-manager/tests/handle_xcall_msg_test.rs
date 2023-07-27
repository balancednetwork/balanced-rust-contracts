mod setup;
use cw_common::{xcall_data_types::Deposit, xcall_msg::XCallMsg as XCallExecuteMsg};
use cw_multi_test::Executor;

use cosmwasm_std::Addr;
use cw_common::{network_address::NetworkAddress, xcall_data_types::DepositRevert};

use crate::setup::{execute_config_x_call, instantiate_contracts};
use cw20::BalanceResponse;
use cw20_base::msg::QueryMsg;
use cw_common::network_address::NetId;
use rlp::{encode, RlpStream};
use setup::{get_event, set_default_connection, setup_context, TestContext};

fn execute_and_handle_message(mut context: TestContext) -> TestContext {
    let cw20_token_addr = context.get_cw20token_app().into_string();
    let call_data = Deposit {
        token_address: cw20_token_addr.clone(),
        from: NetworkAddress("0x01.icon/cx7866543210fedcba9876543210fedcba987654df".to_owned())
            .to_string(),
        to: NetworkAddress("0x01.icon/cx9876543210fedcba9876543210fedcba98765432".to_string())
            .to_string(),
        amount: 1000,
        data: vec![
            118, 101, 99, 33, 91, 49, 44, 32, 50, 44, 32, 51, 44, 32, 52, 44, 32, 53, 93,
        ],
    };
    let _data = encode(&call_data).to_vec();

    let _network_address =
        NetworkAddress("0x01.icon/cx7866543210fedcba9876543210fedcba987654df".to_owned());
    let _sequence_no= 1234u64;
    let message_type: u64 = 1;

    let mut stream = RlpStream::new();
    let method = "Deposit".to_string();
    stream.begin_list(5);
    stream.append(&method);
    stream.append(&cw20_token_addr);
    stream.append(&call_data.from);
    stream.append(&call_data.to);
    stream.append(&call_data.amount);

    let encoded_data: Vec<u8> = stream.out().to_vec();
    print!("Encoded Data {:?}", encoded_data);

    let mut stream = RlpStream::new();
    stream.begin_list(2);
    stream.append(&message_type);
    stream.append(&encoded_data);

    let data = stream.out().to_vec();

    let relay = Addr::unchecked("relay");
    context = set_default_connection(context, relay.clone());

    let response = context
        .app
        .execute_contract(
            relay,
            context.get_xcall_app(),
            &XCallExecuteMsg::HandleMessage {
                from: NetId::from("0x01.icon".to_owned()),
                sn: None,
                msg: data,
            },
            &[],
        )
        .unwrap();

    let event = get_event(&response, "wasm-CallMessage").unwrap();
    let request_id = event.get("reqId").unwrap();
    println!("Request ID {:?}", request_id);

    context
}

#[test]
fn handle_call_message_test() {
    let mut context: TestContext = setup_context();
    context = instantiate_contracts(context);
    context = execute_config_x_call(context, Addr::unchecked("xcall"));
    execute_and_handle_message(context);
}


