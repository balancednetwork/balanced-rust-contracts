use crate::xcall_manager_msg::{
    ProtocolConfig,
    QueryMsg::{GetProtocols, VerifyProtocols},
};
use cosmwasm_std::{to_binary, Addr, DepsMut, QueryRequest, WasmQuery};
use cw_xcall_lib::network_address::NetworkAddress;
use cw_xcall_multi::{error::ContractError, msg::QueryMsg::GetNetworkAddress};

pub fn verify_protocol(
    deps: &DepsMut,
    xcall_manager: Addr,
    protocols: Option<Vec<String>>,
) -> Result<(), ContractError> {
    let query_msg = if let Some(x) = protocols {
        VerifyProtocols { protocols: x }
    } else {
        VerifyProtocols { protocols: vec![] }
    };

    let query = QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: xcall_manager.to_string(),
        msg: to_binary(&query_msg).map_err(ContractError::Std)?,
    });

    let res: bool = deps.querier.query(&query).map_err(ContractError::Std)?;
    if res {
        return Ok(());
    }

    Err(ContractError::Unauthorized {})
}

pub fn get_protocols(deps: &DepsMut, xcall_manager: Addr) -> Result<ProtocolConfig, ContractError> {
    let query_msg = GetProtocols {};
    let query = QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: xcall_manager.to_string(),
        msg: to_binary(&query_msg).map_err(ContractError::Std)?,
    });

    deps.querier.query(&query).map_err(ContractError::Std)
}

pub fn query_network_address(
    deps: &DepsMut,
    x_call_addr: &Addr,
) -> Result<NetworkAddress, ContractError> {
    let query_msg = GetNetworkAddress {};
    let query = QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: x_call_addr.to_string(),
        msg: to_binary(&query_msg).map_err(ContractError::Std)?,
    });

    deps.querier.query(&query).map_err(ContractError::Std)
}
