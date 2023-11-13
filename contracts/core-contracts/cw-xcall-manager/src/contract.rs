use std::collections::HashSet;
use std::hash::Hash;

use cosmwasm_std::{entry_point, CosmosMsg, QuerierWrapper};
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, SubMsg, WasmMsg,
};
use cw2::set_contract_version;

use cw_common::xcall_manager_msg::{
    ConfigureProtocols, Execute, ExecuteMsg, InstantiateMsg, Migrate, MigrateMsg, ProtocolConfig,
    QueryMsg, UpdateAdmin, CONFIGURE_PROTOCOLS, EXECUTE, MIGRATE, UPDATE_ADMIN,
};
use cw_ibc_rlp_lib::rlp::decode;
use cw_ibc_rlp_lib::rlp::Rlp;

use crate::error::ContractError;
use crate::state::*;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-asset-manager";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)
        .map_err(ContractError::Std)?;
    PROPOSER.save(deps.storage, &info.sender)?;
    X_CALL
        .save(deps.storage, &msg.xcall.to_string())
        .map_err(ContractError::Std)?;
    ICON_GOVERNANCE
        .save(deps.storage, &msg.icon_governance)
        .map_err(ContractError::Std)?;
    PROTOCOLS
        .save(deps.storage, &msg.protocols)
        .map_err(ContractError::Std)?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::HandleCallMessage {
            from,
            data,
            protocols,
        } => handle_call_message(deps, info, from, data, protocols),
        ExecuteMsg::ProposeChange { protocol } => {
            let proposer = PROPOSER.load(deps.storage)?;
            if info.sender != proposer {
                return Err(ContractError::OnlyProposer);
            }

            PROPOSED_REMOVAL
                .save(deps.storage, &protocol)
                .map_err(ContractError::Std)?;

            Ok(Response::new())
        }
        ExecuteMsg::RemoveProposal {} => {
            let proposer = PROPOSER.load(deps.storage)?;
            if info.sender != proposer {
                return Err(ContractError::OnlyProposer);
            }

            PROPOSED_REMOVAL.remove(deps.storage);

            Ok(Response::new())
        }
        ExecuteMsg::ChangeProposer { proposer } => {
            let current_proposer = PROPOSER.load(deps.storage)?;
            if info.sender != current_proposer {
                return Err(ContractError::OnlyProposer);
            }

            PROPOSER.save(deps.storage, &proposer)?;
            Ok(Response::new())
        }
    }
}

pub fn handle_call_message(
    deps: DepsMut,
    info: MessageInfo,
    from: String,
    data: Vec<u8>,
    protocols: Option<Vec<String>>,
) -> Result<Response, ContractError> {
    let xcall = X_CALL.load(deps.storage)?;
    if info.sender != xcall {
        return Err(ContractError::OnlyXCall);
    }

    let governance = ICON_GOVERNANCE.load(deps.storage)?;
    if from != governance {
        return Err(ContractError::OnlyGovernance);
    }

    let rlp: Rlp = Rlp::new(&data);
    let method: String = rlp.val_at(0).unwrap();
    let protocols_list = if let Some(x) = protocols { x } else { vec![] };

    let res = verify_protocols(&deps, protocols_list, method.to_string());
    if res.is_err() {
        return Err(res.err().unwrap());
    }

    match method.as_str() {
        CONFIGURE_PROTOCOLS => {
            let configure_protocols: ConfigureProtocols = decode(&data).unwrap();
            for address in &configure_protocols.sources {
                if !is_contract(deps.querier, &Addr::unchecked(address)) {
                    return Err(ContractError::InvalidProtocol);
                }
            }

            let cfg = ProtocolConfig {
                sources: configure_protocols.sources,
                destinations: configure_protocols.destinations,
            };

            PROTOCOLS
                .save(deps.storage, &cfg)
                .map_err(ContractError::Std)?;
            PROPOSED_REMOVAL.remove(deps.storage);
            Ok(Response::new())
        }
        EXECUTE => {
            let execute: Execute = decode(&data).unwrap();
            let wasm_execute_message: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: execute.contract_addr,
                msg: Binary::from_base64(&execute.message)?,
                funds: vec![],
            });

            let sub_message = SubMsg::new(wasm_execute_message);
            Ok(Response::new().add_submessage(sub_message))
        }
        MIGRATE => {
            let migrate: Migrate = decode(&data).unwrap();
            let wasm_execute_message: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Migrate {
                contract_addr: migrate.contract_addr,
                new_code_id: migrate.code_id,
                msg: Binary::from_base64(&migrate.message)?,
            });

            let sub_message = SubMsg::new(wasm_execute_message);
            Ok(Response::new().add_submessage(sub_message))
        }
        UPDATE_ADMIN => {
            let update_admin: UpdateAdmin = decode(&data).unwrap();
            let wasm_execute_message: CosmosMsg = CosmosMsg::Wasm(WasmMsg::UpdateAdmin {
                contract_addr: update_admin.contract_addr,
                admin: update_admin.admin,
            });

            let sub_message = SubMsg::new(wasm_execute_message);
            Ok(Response::new().add_submessage(sub_message))
        }
        _ => Err(ContractError::InvalidMethod),
    }
}

fn verify_protocols(
    deps: &DepsMut,
    protocols: Vec<String>,
    method: String,
) -> Result<(), ContractError> {
    let allowed_protocols = PROTOCOLS.load(deps.storage);
    println!("{:?}", allowed_protocols);
    if allowed_protocols.is_err() {
        return Err(ContractError::NetworkNotConfigured);
    }

    let allowed_protocols = allowed_protocols?.sources;
    if array_eq(&allowed_protocols, &protocols) {
        return Ok(());
    }

    if method != CONFIGURE_PROTOCOLS {
        return Err(ContractError::InvalidProtocol);
    }

    let proposed_protocol_to_remove = PROPOSED_REMOVAL.load(deps.storage)?;
    let joined_protocols = {
        let mut tmp = protocols;
        tmp.push(proposed_protocol_to_remove);
        tmp
    };

    if !array_eq(&joined_protocols, &allowed_protocols) {
        return Err(ContractError::InvalidProtocol);
    }

    Ok(())
}

fn array_eq<T>(a: &[T], b: &[T]) -> bool
where
    T: Eq + Hash,
{
    let a: HashSet<_> = a.iter().collect();
    let b: HashSet<_> = b.iter().collect();

    a == b
}

pub fn is_contract(querier: QuerierWrapper, address: &Addr) -> bool {
    querier.query_wasm_contract_info(address).is_ok()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)
        .map_err(ContractError::Std)?;

    Ok(Response::default().add_attribute("migrate", "successful"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::VerifyProtocols { protocols } => {
            let allowed_protocols = PROTOCOLS.load(deps.storage);
            if allowed_protocols.is_err() {
                return to_binary(&false);
            }

            to_binary(&(allowed_protocols?.sources == protocols))
        }

        QueryMsg::GetProtocols {} => to_binary(&PROTOCOLS.load(deps.storage)?),
    }
}
