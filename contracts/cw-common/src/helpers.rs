use crate::xcall_manager_msg::{
    ProtocolConfig,
    QueryMsg::{GetProtocols, VerifyProtocols},
};
use cosmwasm_std::{
    to_binary, Addr, BalanceResponse, BankQuery, Deps, DepsMut, QueryRequest, WasmQuery,
};
use cw_xcall_lib::network_address::{NetId, NetworkAddress};
use cw_xcall_multi::{
    error::ContractError, msg::QueryMsg::GetFee, msg::QueryMsg::GetNetworkAddress,
};

pub fn verify_protocol(
    deps: &DepsMut,
    xcall_manager: Addr,
    protocols: Option<Vec<String>>,
) -> Result<(), ContractError> {
    let query_msg = VerifyProtocols {
        protocols: protocols.unwrap_or_default(),
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

pub fn get_protocols(deps: &Deps, xcall_manager: Addr) -> Result<ProtocolConfig, ContractError> {
    let query_msg = GetProtocols {};
    let query = QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: xcall_manager.to_string(),
        msg: to_binary(&query_msg).map_err(ContractError::Std)?,
    });

    deps.querier.query(&query).map_err(ContractError::Std)
}

pub fn get_fee(
    deps: &Deps,
    xcall: Addr,
    nid: NetId,
    rollback: bool,
    sources: Option<Vec<String>>,
) -> Result<u128, ContractError> {
    let query_msg = GetFee {
        nid,
        rollback,
        sources,
    };
    let query = QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: xcall.to_string(),
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

pub fn balance_of(deps: &Deps, token: String, owner: String) -> Result<u128, ContractError> {
    let query_msg = cw20::Cw20QueryMsg::Balance { address: owner };
    let query = QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: token,
        msg: to_binary(&query_msg).map_err(ContractError::Std)?,
    });

    let balance_response: cw20::BalanceResponse =
        deps.querier.query(&query).map_err(ContractError::Std)?;
    let balance_u128 = balance_response.balance.u128();
    Ok(balance_u128)
}

pub fn bank_balance_of(deps: &Deps, token: String, owner: String) -> Result<u128, ContractError> {
    let balance_query = BankQuery::Balance {
        address: owner,
        denom: token,
    };
    let balance_response: BalanceResponse = deps.querier.query(&balance_query.into())?;
    let balance_u128 = balance_response.amount.amount.u128();
    Ok(balance_u128)
}
