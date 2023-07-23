#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult, SubMsg,
};
// use cw2::set_contract_version;

use crate::error::ContractError;
use cosmwasm_std::{Reply, StdError};
use cw_common::{
    asset_manager_msg::ExecuteMsg,
    x_call_msg::{InstantiateMsg, XCallMsg, XCallQuery},
};

/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:x-call-mock";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/

const REPLY_MSG_SUCCESS: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: XCallMsg,
) -> Result<Response, ContractError> {
    // match msg {
    //     XCallMsg::SendCallMessage {
    //         to,
    //         data,
    //         rollback,
    //         sources,
    //         destinations,
    //     } => {
    //         print!("to: {}", to.to_string());
    //         print!("data: {:?}", data);
    //         print!("rollback: {:?}", rollback);
    //         print!("sources: {:?}", sources);
    //         print!("destinations: {:?}", destinations);
    //         let _network_address = to;
    //         Ok(Response::default())
    //     }
    //     // XCallMsg::TestHandleCallMessage {
    //     //     from,
    //     //     data,
    //     //     hub_token,
    //     // } => {
    //     //     let call_message = ExecuteMsg::HandleCallMessage {
    //     //         from: cw_common::network_address::NetworkAddress(from),
    //     //         data,
    //     //     };
    //     //     let wasm_execute_message: CosmosMsg = CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
    //     //         contract_addr: hub_token,
    //     //         msg: to_binary(&call_message)?,
    //     //         funds: vec![],
    //     //     });
    //     //     let sub_message = SubMsg::reply_always(wasm_execute_message, REPLY_MSG_SUCCESS);
    //     //     Ok(Response::new()
    //     //         .add_submessage(sub_message)
    //     //         .add_attribute("method", "testhandlecallmessage"))
    //     // }
    //     XCallMsg::SetDefaultConnection { nid: _, address: _ } => todo!(),
    //     XCallMsg::HandleMessage {
    //         from: _,
    //         sn: _,
    //         msg: _,
    //     } => todo!(),
    //     XCallMsg::ExecuteCall {
    //         request_id: _,
    //         data: _,
    //     } => todo!(),
    //     XCallMsg::ExecuteRollback { sequence_no: _ } => todo!(),
    // }
    todo!()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: XCallQuery) -> StdResult<Binary> {
    match _msg {
        XCallQuery::GetNetworkAddress {} => Ok(to_binary("0x05.archway/contract1")?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        REPLY_MSG_SUCCESS => reply_msg_success(deps, env, msg),
        _ => Err(ContractError::Std(StdError::generic_err(
            "reply id not found",
        ))),
    }
}

pub fn reply_msg_success(_deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.result {
        cosmwasm_std::SubMsgResult::Ok(_) => Ok(Response::default()),
        cosmwasm_std::SubMsgResult::Err(error) => {
            Err(StdError::GenericErr { msg: error }).map_err(Into::<ContractError>::into)?
        }
    }
}

#[cfg(test)]
mod tests {}
