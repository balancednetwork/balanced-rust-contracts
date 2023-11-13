use cosmwasm_std::testing::{
    mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, ContractInfoResponse, ContractResult, CosmosMsg, Deps,
    Empty, OwnedDeps, SystemResult, WasmMsg, WasmQuery,
};
use cw_common::asset_manager_msg::{
    ExecuteMsg as AssetManagerExecuteMessage, MigrateMsg as AssetManageMigrateMsg,
};
use cw_common::xcall_manager_msg::{
    ConfigureProtocols, Execute, ExecuteMsg, InstantiateMsg, Migrate, ProtocolConfig, QueryMsg,
    UpdateAdmin,
};
use cw_ibc_rlp_lib::rlp::encode;
use cw_xcall_manager::contract::{execute, instantiate, query};
use cw_xcall_manager::ContractError;

pub const XCALL_NETWORK_ADDRESS: &str = "archway/xcall";
pub const XCALL_ADDR: &str = "xcall";
pub const GOVERNANCE: &str = "icon/governance";
pub const PROPOSER: &str = "proposer";

fn setup(
    mut deps: OwnedDeps<MockStorage, MockApi, MockQuerier, Empty>,
    protocols: Vec<String>,
) -> OwnedDeps<MockStorage, MockApi, MockQuerier, Empty> {
    deps.querier.update_wasm(|r: &WasmQuery| match r {
        WasmQuery::Smart {
            contract_addr: _,
            msg: _,
        } => SystemResult::Ok(ContractResult::Ok(
            to_binary(&XCALL_NETWORK_ADDRESS).unwrap(),
        )),
        WasmQuery::ContractInfo { contract_addr: _ } => SystemResult::Ok(ContractResult::Ok(
            to_binary(&ContractInfoResponse::default()).unwrap(),
        )),
        _ => todo!(),
    });

    let msg = InstantiateMsg {
        xcall: Addr::unchecked("xcall"),
        icon_governance: GOVERNANCE.to_string(),
        protocols: ProtocolConfig {
            sources: protocols.clone(),
            destinations: protocols,
        },
    };

    let info = mock_info(PROPOSER, &[]);
    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps.as_mut(), mock_env(), info, msg);
    assert!(res.is_ok());
    deps
}

#[test]
fn verify_protocol_empty() {
    // Arrange
    let mut deps = mock_dependencies();
    deps = setup(deps, vec![]);

    // Act
    verify_protocol(deps.as_ref(), vec![], true);
}

#[test]
fn verify_protocol_base() {
    // Arrange
    let mut deps = mock_dependencies();
    let protocols = vec!["Protocol1".to_string(), "Protocol2".to_string()];
    deps = setup(deps, protocols.clone());

    // Act & Assert
    verify_protocol(deps.as_ref(), protocols, true);
}

#[test]
fn verify_protocol_invalid() {
    // Arrange
    let mut deps = mock_dependencies();
    let protocols = vec!["Protocol1".to_string(), "Protocol2".to_string()];
    let invalid_protocols = vec!["Protocol3".to_string(), "Protocol2".to_string()];
    deps = setup(deps, protocols);

    // Act
    verify_protocol(deps.as_ref(), invalid_protocols, false);
}

#[test]
fn verify_protocol_invalid_empty() {
    // Arrange
    let mut deps = mock_dependencies();
    let protocols = vec!["Protocol1".to_string()];
    deps = setup(deps, vec![]);

    // Act & Assert
    verify_protocol(deps.as_ref(), protocols, false);
}

#[test]
fn verify_only_xcall() {
    // Arrange
    let mut deps = mock_dependencies();
    deps = setup(deps, vec![]);

    let msg = ExecuteMsg::HandleCallMessage {
        from: GOVERNANCE.to_string(),
        data: vec![],
        protocols: None,
    };

    // Act
    let res = execute(deps.as_mut(), mock_env(), mock_info("not_xcall", &[]), msg);

    // Assert
    assert!(res.is_err() && res.unwrap_err() == ContractError::OnlyXCall);
}

#[test]
fn verify_only_governance() {
    // Arrange
    let mut deps = mock_dependencies();
    deps = setup(deps, vec![]);

    let non_governance = "icon/fake_governance".to_string();
    let msg = ExecuteMsg::HandleCallMessage {
        from: non_governance,
        data: vec![],
        protocols: None,
    };

    // Act
    let res = execute(deps.as_mut(), mock_env(), mock_info(XCALL_ADDR, &[]), msg);

    // Assert
    assert!(res.is_err() && res.unwrap_err() == ContractError::OnlyGovernance);
}

#[test]
fn configure_protocol() {
    // Arrange
    let mut deps = mock_dependencies();
    let protocols = vec!["Protocol1".to_string()];
    let new_sources = vec!["Protocol2".to_string()];
    let new_destination = vec!["icon/dst".to_string()];
    deps = setup(deps, protocols.clone());

    let xcall_message = ConfigureProtocols {
        sources: new_sources.clone(),
        destinations: new_destination.clone(),
    };
    let encoded_message = encode(&xcall_message).to_vec();
    let msg = ExecuteMsg::HandleCallMessage {
        from: GOVERNANCE.to_string(),
        data: encoded_message,
        protocols: Some(protocols.clone()),
    };

    // Act
    let res = execute(deps.as_mut(), mock_env(), mock_info(XCALL_ADDR, &[]), msg);

    // Assert
    assert!(res.is_ok());

    verify_protocol(deps.as_ref(), protocols, false);
    verify_protocol(deps.as_ref(), new_sources.clone(), true);
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetProtocols {});
    let current_protocols: ProtocolConfig = from_binary(&res.unwrap()).unwrap();
    assert!(current_protocols.sources == new_sources);
    assert!(current_protocols.destinations == new_destination);
}

#[test]
fn execute_message() {
    // Arrange
    let mut deps = mock_dependencies();
    let protocols = vec!["Protocol1".to_string()];
    deps = setup(deps, protocols.clone());

    let to = "contract".to_string();
    let execute_message = AssetManagerExecuteMessage::ConfigureNative {
        native_token_address: "a".to_string(),
        native_token_manager: "b".to_string(),
    };
    let xcall_message = Execute {
        contract_addr: to.clone(),
        message: Binary::to_base64(&to_binary(&execute_message).unwrap()),
    };
    let expected_message = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: to,
        msg: to_binary(&execute_message).unwrap(),
        funds: vec![],
    });
    let encoded_message = encode(&xcall_message).to_vec();
    let msg = ExecuteMsg::HandleCallMessage {
        from: GOVERNANCE.to_string(),
        data: encoded_message,
        protocols: Some(protocols),
    };

    // Act
    let res = execute(deps.as_mut(), mock_env(), mock_info(XCALL_ADDR, &[]), msg);

    // Assert
    assert!(res.is_ok() && res.unwrap().messages[0].msg.eq(&expected_message));
}

#[test]
fn migrate_message() {
    // Arrange
    let mut deps = mock_dependencies();
    let protocols = vec!["Protocol1".to_string()];
    deps = setup(deps, protocols.clone());

    let to = "contract".to_string();
    let code_id = 2;
    let migrate_message = AssetManageMigrateMsg {};
    let xcall_message = Migrate {
        contract_addr: to.clone(),
        code_id,
        message: Binary::to_base64(&to_binary(&migrate_message).unwrap()),
    };

    let expected_message = CosmosMsg::Wasm(WasmMsg::Migrate {
        contract_addr: to,
        new_code_id: code_id,
        msg: to_binary(&migrate_message).unwrap(),
    });

    let encoded_message = encode(&xcall_message).to_vec();
    let msg = ExecuteMsg::HandleCallMessage {
        from: GOVERNANCE.to_string(),
        data: encoded_message,
        protocols: Some(protocols),
    };

    // Act
    let res = execute(deps.as_mut(), mock_env(), mock_info(XCALL_ADDR, &[]), msg);

    // Assert
    assert!(res.is_ok() && res.unwrap().messages[0].msg.eq(&expected_message));
}

#[test]
fn update_admin() {
    // Arrange
    let mut deps = mock_dependencies();
    let protocols = vec!["Protocol1".to_string()];
    deps = setup(deps, protocols.clone());

    let to = "contract".to_string();
    let admin = "admin".to_string();
    let xcall_message = UpdateAdmin {
        contract_addr: to.clone(),
        admin: admin.clone(),
    };

    let expected_message = CosmosMsg::Wasm(WasmMsg::UpdateAdmin {
        contract_addr: to,
        admin,
    });

    let encoded_message = encode(&xcall_message).to_vec();
    let msg = ExecuteMsg::HandleCallMessage {
        from: GOVERNANCE.to_string(),
        data: encoded_message,
        protocols: Some(protocols),
    };

    // Act
    let res = execute(deps.as_mut(), mock_env(), mock_info(XCALL_ADDR, &[]), msg);

    // Assert
    assert!(res.is_ok() && res.unwrap().messages[0].msg.eq(&expected_message));
}

#[test]
fn only_proposer() {
    // Arrange
    let mut deps = mock_dependencies();
    deps = setup(deps, vec![]);

    // Act & Assert
    let msg = ExecuteMsg::ProposeChange {
        protocol: "a".to_string(),
    };
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("not_proposer", &[]),
        msg,
    );
    assert!(res.is_err() && res.unwrap_err() == ContractError::OnlyProposer);

    // Act & Assert
    let msg = ExecuteMsg::RemoveProposal {};
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("not_proposer", &[]),
        msg,
    );
    assert!(res.is_err() && res.unwrap_err() == ContractError::OnlyProposer);

    // Act & Assert
    let msg = ExecuteMsg::ChangeProposer {
        proposer: Addr::unchecked("a".to_string()),
    };
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("not_proposer", &[]),
        msg,
    );
    assert!(res.is_err() && res.unwrap_err() == ContractError::OnlyProposer);
}

#[test]
fn propose_and_enact_removal() {
    // Arrange
    let mut deps = mock_dependencies();
    let protocols = vec!["Protocol1".to_string(), "Protocol2".to_string()];
    let new_protocols = vec!["Protocol2".to_string()];
    deps = setup(deps, protocols.clone());

    let msg = ExecuteMsg::ProposeChange {
        protocol: protocols[0].clone(),
    };
    let res = execute(deps.as_mut(), mock_env(), mock_info(PROPOSER, &[]), msg);
    assert!(res.is_ok());

    let xcall_message = ConfigureProtocols {
        sources: new_protocols.clone(),
        destinations: new_protocols.clone(),
    };
    let encoded_message = encode(&xcall_message).to_vec();
    let msg = ExecuteMsg::HandleCallMessage {
        from: GOVERNANCE.to_string(),
        data: encoded_message,
        protocols: Some(new_protocols.clone()),
    };

    // Act
    let res = execute(deps.as_mut(), mock_env(), mock_info(XCALL_ADDR, &[]), msg);

    // Assert
    assert!(res.is_ok());

    verify_protocol(deps.as_ref(), protocols, false);
    verify_protocol(deps.as_ref(), new_protocols, true)
}

#[test]
fn invalid_proposal() {
    // Arrange
    let mut deps = mock_dependencies();
    let protocols = vec!["Protocol1".to_string(), "Protocol2".to_string()];
    let new_protocols = vec!["Protocol2".to_string()];
    deps = setup(deps, protocols);

    let msg = ExecuteMsg::ProposeChange {
        protocol: "Protocol3".to_string(),
    };
    let res = execute(deps.as_mut(), mock_env(), mock_info(PROPOSER, &[]), msg);
    assert!(res.is_ok());

    let xcall_message = ConfigureProtocols {
        sources: new_protocols.clone(),
        destinations: new_protocols.clone(),
    };
    let encoded_message = encode(&xcall_message).to_vec();
    let msg = ExecuteMsg::HandleCallMessage {
        from: GOVERNANCE.to_string(),
        data: encoded_message,
        protocols: Some(new_protocols),
    };

    // Act
    let res = execute(deps.as_mut(), mock_env(), mock_info(XCALL_ADDR, &[]), msg);

    // Assert
    assert!(res.is_err() && res.unwrap_err() == ContractError::InvalidProtocol);
}

#[test]
fn change_proposer() {
    // Arrange
    let mut deps = mock_dependencies();
    deps = setup(deps, vec![]);
    let new_proposer = Addr::unchecked("new proposer".to_string());

    let msg = ExecuteMsg::ChangeProposer {
        proposer: new_proposer.clone(),
    };

    // Act
    let res = execute(deps.as_mut(), mock_env(), mock_info(PROPOSER, &[]), msg);

    // Assert
    assert!(res.is_ok());
    let msg = ExecuteMsg::ProposeChange {
        protocol: "Protocol1".to_string(),
    };
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(new_proposer.as_ref(), &[]),
        msg,
    );
    assert!(res.is_ok());
}

fn verify_protocol(deps: Deps<'_, Empty>, protocols: Vec<String>, valid: bool) {
    let res = query(
        deps,
        mock_env(),
        QueryMsg::VerifyProtocols {
            protocols: protocols.clone(),
        },
    );
    assert!(res.is_ok());
    assert!(res.unwrap() == to_binary(&valid).unwrap());
    if !valid {
        return;
    }

    let res = query(deps, mock_env(), QueryMsg::GetProtocols {});
    let current_protocols: ProtocolConfig = from_binary(&res.unwrap()).unwrap();
    assert!(current_protocols.sources == protocols);
}
