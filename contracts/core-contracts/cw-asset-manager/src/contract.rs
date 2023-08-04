use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, Event, MessageInfo, QueryRequest, Reply, Response,
    StdError, StdResult, SubMsg, SubMsgResult, Uint128, WasmMsg, WasmQuery,
};
// use cw2::set_contract_version;
use cw20::{AllowanceResponse, Cw20ExecuteMsg, Cw20QueryMsg};

use cw_common::asset_manager_msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use cw_common::network_address::NetworkAddress;
use cw_common::x_call_msg::{XCallExecuteMsg, XCallQuery};
use cw_common::xcall_data_types::Deposit;

use crate::constants::SUCCESS_REPLY_MSG;
use crate::error::ContractError;
use crate::helpers::{decode_encoded_bytes, validate_archway_address, DecodedStruct};
use crate::state::*;

// // version info for migration info
// const CONTRACT_NAME: &str = "crates.io:cw-asset-manager";
// const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    // set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    OWNER.save(deps.storage, &info.sender)?;
    Ok(Response::new().add_attribute("action", "instantiated"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ConfigureXcall {
            source_xcall,
            destination_asset_manager,
        } => exec::configure_network(deps, info, source_xcall, destination_asset_manager),
        ExecuteMsg::HandleCallMessage { from, data } => {
            exec::handle_xcall_msg(deps, env, info, from, data)
        }

        ExecuteMsg::Deposit {
            token_address,
            amount,
            to,
            data,
        } => {
            let nid = NID.load(deps.storage)?;
            let depositor = NetworkAddress::new(nid.as_str(), info.sender.as_str());
            // Performing necessary validation and logic for the Deposit variant
            let (_, is_valid_address) = validate_archway_address(&deps, &token_address);
            if !is_valid_address {
                return Err(ContractError::InvalidTokenAddress);
            }

            if amount.is_zero() {
                return Err(ContractError::InvalidAmount);
            }

            //you can optimize this
            let recipient: NetworkAddress = match to {
                Some(to_address) => {
                    let nw_addr = NetworkAddress(to_address);
                    if !nw_addr.validate() {
                        return Err(ContractError::InvalidRecipientAddress);
                    }
                    nw_addr
                }
                None => depositor.clone(),
            };

            //if nw_addr validation is not required
            //alternative: let recipient = to.unwrap_or_else(|| info.sender.to_String());

            // we can Perform additional logic based on the to field later
            let data = data.unwrap_or_default();

            let res = exec::deposit_cw20_tokens(
                deps,
                env,
                token_address,
                depositor,
                amount,
                recipient,
                data,
            )?;
            Ok(res)
        }
    }
}

mod exec {
    use rlp::Encodable;

    use cw_common::xcall_data_types::DepositRevert;

    use super::*;

    pub fn configure_network(
        deps: DepsMut,
        info: MessageInfo,
        source_xcall: String,
        destination_asset_manager: String,
    ) -> Result<Response, ContractError> {
        let owner = OWNER.load(deps.storage)?;
        if info.sender != owner {
            return Err(ContractError::OnlyOwner);
        }

        let x_addr = deps.api.addr_validate(source_xcall.as_ref())?;

        let query_msg = XCallQuery::GetNetworkAddress {};

        let query = QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: x_addr.to_string(),
            msg: to_binary(&query_msg)?,
        });

        let x_network_address: NetworkAddress = deps.querier.query(&query)?;

        if x_network_address.is_empty() {
            return Err(ContractError::XAddressNotFound);
        }

        let (nid, _) = x_network_address.parse_parts();

        let dest_nw_addr = NetworkAddress(destination_asset_manager);

        if !dest_nw_addr.validate() {
            return Err(ContractError::InvalidNetworkAddressFormat {});
        }

        //save incase required
        let (dest_id, _dest_address) = dest_nw_addr.parse_parts();

        //update state
        X_NETWORK_ADDRESS.save(deps.storage, &x_network_address)?;
        NID.save(deps.storage, &nid)?;
        //TODO: Rename to ICON asset manager
        ICON_ASSET_MANAGER.save(deps.storage, &dest_nw_addr)?;
        SOURCE_XCALL.save(deps.storage, &source_xcall)?;
        ICON_ASSET_MANAGER.save(deps.storage, &dest_nw_addr)?;
        ICON_NET_ID.save(deps.storage, &dest_id)?;

        //TODO: save the details
        Ok(Response::default())
    }

    pub fn deposit_cw20_tokens(
        deps: DepsMut,
        env: Env,
        token_address: String,
        from: NetworkAddress,
        amount: Uint128,
        to: NetworkAddress,
        data: Vec<u8>,
    ) -> Result<Response, ContractError> {
        let token = deps.api.addr_validate(&token_address)?;
        let dest_am = ICON_ASSET_MANAGER.load(deps.storage)?;

        let contract_address = &env.contract.address;

        let query_msg = &Cw20QueryMsg::Allowance {
            owner: from.account().to_string(),
            spender: contract_address.to_string(),
        };

        let query_resp: AllowanceResponse = deps
            .querier
            .query_wasm_smart::<AllowanceResponse>(token_address.clone(), &query_msg)?;

        //check allowance
        if query_resp.allowance < amount {
            //TODO: create specific error
            return Err(ContractError::InsufficientTokenAllowance {});
        }

        let transfer_token_msg = to_binary(&Cw20ExecuteMsg::TransferFrom {
            owner: from.account().to_string(),
            recipient: contract_address.into(),
            amount,
        })?;

        let execute_msg = WasmMsg::Execute {
            contract_addr: token_address.to_owned(),
            msg: transfer_token_msg,
            funds: vec![],
        };

        //transfer sub msg
        let transfer_sub_msg = SubMsg::reply_always(execute_msg, SUCCESS_REPLY_MSG);

        //create xcall rlp encode data
        let xcall_data = Deposit {
            token_address: token_address.to_owned(),
            from: from.account().to_string(),
            to: to.to_string(),
            amount: Uint128::u128(&amount),
            data,
        };

        let source_xcall = SOURCE_XCALL.load(deps.storage)?;
        //create xcall msg for dispatching  send call
        let xcall_message = XCallExecuteMsg::SendCallMessage {
            to: dest_am.to_string(),
            data: xcall_data.rlp_bytes().to_vec(),
            //TODO: add the rollback with deposit revert information
            rollback: Some(
                DepositRevert {
                    token_address,
                    account: from.account().to_string(),
                    amount: Uint128::u128(&amount),
                }
                .rlp_bytes()
                .to_vec(),
            ),

            sources: None,
            destinations: None,
        };

        let xcall_msg = WasmMsg::Execute {
            contract_addr: source_xcall,
            msg: to_binary(&xcall_message)?,
            funds: vec![],
        };

        let xcall_sub_msg = SubMsg::reply_always(xcall_msg, SUCCESS_REPLY_MSG);

        let attributes = vec![
            ("Token", token.to_string()),
            ("To", to.to_string()),
            ("Amount", amount.to_string()),
        ];

        let event = Event::new("Deposit").add_attributes(attributes);

        let resp = Response::new()
            .add_submessages(vec![transfer_sub_msg, xcall_sub_msg])
            .add_event(event);

        Ok(resp)
    }

    pub fn reply_handler(msg: SubMsgResult) -> Result<Response, ContractError> {
        let result = msg.into_result();
        match result {
            Ok(_) => Ok(Response::default()),
            Err(err) => Err(StdError::generic_err(err).into()),
        }
    }

    pub fn handle_xcall_msg(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        from: String,
        data: Vec<u8>,
    ) -> Result<Response, ContractError> {
        let xcall = SOURCE_XCALL.load(deps.storage)?;
        let x_call_addr = deps.api.addr_validate(&xcall)?;
        let x_network = X_NETWORK_ADDRESS.load(deps.storage)?;

        if info.sender != x_call_addr {
            return Err(ContractError::OnlyXcallService);
        }

        let (_, decoded_struct) = decode_encoded_bytes(&data)?;

        let res = match decoded_struct {
            DecodedStruct::DepositRevert(data) => {
                //TODO: _from should be with network address of xcall in archway
                if from != x_network.to_string() {
                    return Err(ContractError::FailedXcallNetworkMatch);
                }

                let token_address = data.token_address;
                let account = data.account;
                let amount = Uint128::from(data.amount);

                transfer_tokens(deps, account, token_address, amount)?
            }

            DecodedStruct::WithdrawTo(data_struct) => {
                //TODO: Check if _from is ICON Asset manager contract
                let icon_am = ICON_ASSET_MANAGER.load(deps.storage)?;
                if from != icon_am.to_string() {
                    return Err(ContractError::OnlyIconAssetManager {});
                }

                let token_address = data_struct.token_address;
                let account = data_struct.user_address;
                let amount = Uint128::from(data_struct.amount);

                transfer_tokens(deps, account, token_address, amount)?
            }
        };

        Ok(res)
    }

    //internal function to transfer tokens from contract to account
    pub fn transfer_tokens(
        deps: DepsMut,
        account: String,
        token_address: String,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        deps.api.addr_validate(&account)?;
        deps.api.addr_validate(&token_address)?;

        let transfer_msg = &Cw20ExecuteMsg::Transfer {
            recipient: account,
            amount,
        };

        let execute_msg = WasmMsg::Execute {
            contract_addr: token_address,
            msg: to_binary(transfer_msg)?,
            funds: vec![],
        };

        let sub_msg = SubMsg::reply_always(execute_msg, SUCCESS_REPLY_MSG);
        Ok(Response::new().add_submessage(sub_msg))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetOwner {} => to_binary(&query::query_get_owner(deps)?),
        QueryMsg::GetConfiguration {} => to_binary(&query::query_conifg(deps)?),
        QueryMsg::GetNetIds {} => to_binary(&query::query_nid(deps)?),
    }
}

mod query {
    use super::*;
    use cw_common::asset_manager_msg::{ConfigureResponse, NetIdResponse, OwnerResponse};

    pub fn query_get_owner(deps: Deps) -> StdResult<OwnerResponse> {
        let owner = OWNER.load(deps.storage)?;
        Ok(OwnerResponse { owner })
    }

    pub fn query_conifg(deps: Deps) -> StdResult<ConfigureResponse> {
        let source_x_call = SOURCE_XCALL.load(deps.storage)?;
        let source_xcall = Addr::unchecked(source_x_call);
        let icon_asset_manager = (ICON_ASSET_MANAGER.load(deps.storage)?).to_string();

        Ok(ConfigureResponse {
            source_xcall,
            icon_asset_manager,
        })
    }

    pub fn query_nid(deps: Deps) -> StdResult<NetIdResponse> {
        let x_call_nid = NID.load(deps.storage)?.to_string();
        let icon_nid = ICON_NET_ID.load(deps.storage)?.to_string();

        Ok(NetIdResponse {
            x_call_nid,
            icon_nid,
        })
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        SUCCESS_REPLY_MSG => exec::reply_handler(msg.result),
        _ => Err(StdError::generic_err("unknown reply id"))?,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::exec::configure_network;

    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier},
        ContractResult, MemoryStorage, OwnedDeps, SystemResult, Uint128, WasmQuery,
    };
    use rlp::Encodable;

    use cw_common::xcall_data_types::DepositRevert;
    use cw_common::{asset_manager_msg::InstantiateMsg, xcall_data_types::WithdrawTo};

    //similar to fixtures
    fn test_setup() -> (
        OwnedDeps<MemoryStorage, MockApi, MockQuerier>,
        Env,
        MessageInfo,
        Response,
    ) {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("user", &[]);

        let instantiated_resp =
            instantiate(deps.as_mut(), env.clone(), info.clone(), InstantiateMsg {}).unwrap();

        //to pretend us as xcall contract during handle call execution testing
        let xcall = "xcall";

        let configure_msg = ExecuteMsg::ConfigureXcall {
            source_xcall: xcall.to_owned(),
            destination_asset_manager: "0x01.icon/cxc2d01de5013778d71d99f985e4e2ff3a9b48a66c"
                .to_owned(),
        };

        // mocking response for external query i.e. allowance
        deps.querier.update_wasm(|r: &WasmQuery| match r {
            WasmQuery::Smart {
                contract_addr,
                msg: _,
            } => {
                if contract_addr == &xcall.to_owned() {
                    SystemResult::Ok(ContractResult::Ok(
                        to_binary(&"0x44.archway/xcall".to_owned()).unwrap(),
                    ))
                } else {
                    //mock allowance resp
                    let allowance_resp = AllowanceResponse {
                        allowance: Uint128::new(1000),
                        expires: cw_utils::Expiration::Never {},
                    };
                    SystemResult::Ok(ContractResult::Ok(to_binary(&allowance_resp).unwrap()))
                }
            }
            _ => todo!(),
        });

        execute(deps.as_mut(), env.clone(), info.clone(), configure_msg).unwrap();

        (deps, env, info, instantiated_resp)
    }

    #[test]
    fn test_instantiate() {
        let (deps, _, info, res) = test_setup();

        //check proper instantiation
        assert_eq!(res.attributes.len(), 1);
        assert_eq!(res.attributes[0], ("action", "instantiated"));

        let owner = OWNER.load(&deps.storage).unwrap();
        assert_eq!(owner, info.sender);
    }

    #[test]
    fn test_deposit_for_sufficient_allowance() {
        let (mut deps, env, info, _) = test_setup();

        let destination_asset_manager = ICON_ASSET_MANAGER.load(deps.as_ref().storage).unwrap();
        assert_eq!(
            destination_asset_manager.to_string(),
            "0x01.icon/cxc2d01de5013778d71d99f985e4e2ff3a9b48a66c".to_string()
        );

        // Test Deposit message (checking expected field value)
        let msg = ExecuteMsg::Deposit {
            token_address: "token1".to_string(),
            amount: Uint128::new(100),
            to: None,
            data: None,
        };

        let response = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Verify the response contains the expected sub-messages
        assert_eq!(response.messages.len(), 2);

        // Verify the event attributes
        if let Some(event) = response.events.get(0) {
            assert_eq!(event.ty, "Deposit");
            assert_eq!(event.attributes.len(), 3);

            // Verify the individual event attributes
            for attribute in &event.attributes {
                match attribute.key.as_str() {
                    "Token" => assert_eq!(attribute.value, "token1"),
                    "To" => assert_eq!(attribute.value, "0x44.archway/user"),
                    "Amount" => assert_eq!(attribute.value, "100"),
                    _ => panic!("Unexpected attribute key"),
                }
            }
        } else {
            panic!("No event found in the response");
        }

        //check for some address for to field
        let msg = ExecuteMsg::Deposit {
            token_address: "token1".to_string(),
            amount: Uint128::new(100),
            to: Some(String::from(
                "0x01.icon/cx9876543210fedcba9876543210fedcba98765432",
            )),
            data: None,
        };

        let result = execute(deps.as_mut(), env, info, msg).unwrap();
        for attribute in &result.events[0].attributes {
            match attribute.key.as_str() {
                "Token" => assert_eq!(attribute.value, "token1"),
                "To" => println!("value: {:?}", attribute.value),
                "Amount" => assert_eq!(attribute.value, "100"),
                _ => panic!("Unexpected attribute key"),
            }
        }
    }

    #[test]
    fn test_deposit_for_insufficient_allowance() {
        let (mut deps, env, info, _) = test_setup();

        let destination_asset_manager = ICON_ASSET_MANAGER.load(deps.as_ref().storage).unwrap();
        assert_eq!(
            destination_asset_manager.to_string(),
            "0x01.icon/cxc2d01de5013778d71d99f985e4e2ff3a9b48a66c".to_string()
        );

        // Test Deposit message
        let msg = ExecuteMsg::Deposit {
            token_address: "token1".to_string(),
            amount: Uint128::new(1500),
            to: None,
            data: None,
        };

        let result = execute(deps.as_mut(), env, info, msg);
        assert!(result.is_err());
    }

    #[test]
    #[should_panic]
    fn test_deposit_for_invalid_zero_amount() {
        let (mut deps, env, info, _) = test_setup();

        let destination_asset_manager = ICON_ASSET_MANAGER.load(deps.as_ref().storage).unwrap();
        assert_eq!(
            destination_asset_manager.to_string(),
            "0x01.icon/cxc2d01de5013778d71d99f985e4e2ff3a9b48a66c".to_string()
        );

        // Test Deposit message
        let msg = ExecuteMsg::Deposit {
            token_address: "token1".to_string(),
            amount: Uint128::new(0),
            to: None,
            data: None,
        };

        execute(deps.as_mut(), env, info, msg).unwrap();
    }

    #[test]
    fn test_handle_xcall() {
        let (mut deps, env, _, _) = test_setup();
        let mocked_xcall_info = mock_info("xcall", &[]);

        let xcall_nw = "0x44.archway/xcall";
        let token = "token1";
        let account = "account1";
        //create deposit revert(expected)  xcall msg deps
        let x_deposit_revert = DepositRevert {
            token_address: token.to_string(),
            account: account.to_string(),
            amount: 100,
        };

        //create valid handle_call_message
        let msg = ExecuteMsg::HandleCallMessage {
            from: xcall_nw.to_string(),
            data: x_deposit_revert.rlp_bytes().to_vec(),
        };

        let result = execute(deps.as_mut(), env.clone(), mocked_xcall_info.clone(), msg);

        //check for valid xcall expected msg data

        assert!(result.is_ok());

        //for withdrawTo
        let am_nw = "0x01.icon/cxc2d01de5013778d71d99f985e4e2ff3a9b48a66c";
        let withdraw_msg = WithdrawTo {
            token_address: token.to_string(),
            amount: 1000,
            user_address: account.to_string(),
        };

        let exe_msg = ExecuteMsg::HandleCallMessage {
            from: am_nw.to_string(),
            data: withdraw_msg.rlp_bytes().to_vec(),
        };
        let resp = execute(
            deps.as_mut(),
            env.clone(),
            mocked_xcall_info.clone(),
            exe_msg,
        );
        assert!(resp.is_ok());

        //----------------------------------------------//
        //check for unhandled xcall msg data
        //----------------------------------------------//

        let x_msg = Deposit {
            token_address: String::from("token1"),
            from: String::from("userrrr"),
            amount: 100,
            to: String::from("account1"),
            data: vec![],
        };

        let unknown_msg = ExecuteMsg::HandleCallMessage {
            from: xcall_nw.to_string(),
            data: x_msg.rlp_bytes().to_vec(),
        };

        //check for error due to unknown xcall handle data
        let result = execute(deps.as_mut(), env, mocked_xcall_info, unknown_msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_configure_network() {
        //verify configuration updates from owner side
        let (mut deps, env, info, _) = test_setup();

        let source_xcall = "xcall".to_string();
        let destination_asset_manager =
            "0x01.icon/hx9876543210fedcba9876543210fedcba98765432".to_string();
        // Execute the function
        let msg = ExecuteMsg::ConfigureXcall {
            source_xcall: source_xcall.to_owned(),
            destination_asset_manager: destination_asset_manager.to_owned(),
        };

        let res = execute(deps.as_mut(), env, info, msg);

        // Check the response
        assert!(res.is_ok());
        let response: Response = res.unwrap();
        assert_eq!(response, Response::default());

        // Verify the saved values
        let saved_source_xcall: String = SOURCE_XCALL.load(deps.as_ref().storage).unwrap();
        let icon_am = ICON_ASSET_MANAGER.load(deps.as_ref().storage).unwrap();
        let saved_destination_asset_manager = icon_am.to_string();

        assert_eq!(saved_source_xcall, source_xcall);
        assert_eq!(saved_destination_asset_manager, destination_asset_manager);

        // Verify that only the owner can configure the network
        let other_info = mock_info("other_sender", &[]);
        let res = configure_network(
            deps.as_mut(),
            other_info,
            source_xcall,
            destination_asset_manager,
        );

        //check for error
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert_eq!(err, ContractError::OnlyOwner);
    }
}
