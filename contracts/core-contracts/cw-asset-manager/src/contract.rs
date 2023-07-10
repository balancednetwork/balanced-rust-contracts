use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, Event, MessageInfo, Reply, Response, StdError,
    StdResult, SubMsg, SubMsgResult, Uint128, WasmMsg,QueryRequest,WasmQuery
};
// use cw2::set_contract_version;
use cw20::{Cw20ExecuteMsg, Cw20QueryMsg};

use cw_common::asset_manager_msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use cw_common::xcall_data_types::Deposit;
use cw_common::xcall_msg::{XCallMsg,XCallQuery};
use cw_common::network_address::{NetworkAddress,NetId};

use crate::constants::SUCCESS_REPLY_MSG;
use crate::error::ContractError;
use crate::helpers::{decode_encoded_bytes, DecodedStruct,validate_archway_address};
use crate::state::*;

// // version info for migration info
// const CONTRACT_NAME: &str = "crates.io:cw-asset-manager";
// const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
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
            data
        } => {
                 // Performing necessary validation and logic for the Deposit variant
             let (_,isValidAddress) = validate_archway_address(&deps, &token_address);
            if !isValidAddress {
                return Err(ContractError::InvalidTokenAddress);
            }

            if amount.is_zero() {
                return Err(ContractError::InvalidAmount);
            }
            
            //you can optimize this 
            let recipient = match to {
                Some(to_address) => {
                    let nw_addr = NetworkAddress(to_address.clone());
                    if !nw_addr.validate() {
                        return Err(ContractError::InvalidRecipientAddress);
                    }
                    to_address
                }
                None => info.sender.to_string(),
            };

            //if nw_addr validation is not required
            //alternative: let recipient = to.unwrap_or_else(|| info.sender.to_String());
        
            
            // we can Perform additional logic based on the to field later
            let data = if let Some(data) = data {
                data
            } else {
                Vec::<u8>::new()
            };

           
            exec::deposit_cw20_tokens(deps, info, env, token_address, amount, recipient,data)?;
            Ok(Response::default())
        },
        
  
}
}


mod exec {
    use rlp::Encodable;
    use std::str::FromStr;

    use cw_common::xcall_data_types::{DepositRevert, WithdrawRequest};

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
        
        let query_msg = XCallQuery::GetNetworkAddress { };

       
        let query = QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: source_xcall.clone(),
             msg: to_binary(&query_msg)?
            });

        let x_network_address:NetworkAddress =  deps.querier.query(&query)?;

        if x_network_address.is_empty() {
            return Err(ContractError::XAddressNotFound)
        }

        let (nid,address) = x_network_address.parse_parts();
        if x_addr != address {
            return Err(ContractError::FailedXaddressCheck{})
        }

        let dest_nw_addr =NetworkAddress(destination_asset_manager);
        if !dest_nw_addr.validate() {
            return  Err(ContractError::InvalidNetworkAddressFormat{})
        }

        //save incase required
        let (dest_id,dest_address) = dest_nw_addr.parse_parts();

        //update state
        X_NETWORK_ADDRESS.save(deps.storage, &x_network_address)?;
        NID.save(deps.storage,&nid)?;
        //TODO: Rename to ICON asset manager
        ICON_ASSET_MANAGER.save(deps.storage, &dest_nw_addr)?;

        //TODO: verify both addresses, verify for archway, verify for network address
        SOURCE_XCALL.save(deps.storage, &source_xcall)?;
        ICON_ASSET_MANAGER.save(deps.storage, &dest_nw_addr)?;
        //TODO: save the details
        Ok(Response::default())
    }

    pub fn deposit_cw20_tokens(
        deps: DepsMut,
        info: MessageInfo,
        env: Env,
        token_address: String,
        amount: Uint128,
        to: String,
        data: Vec<u8>
    ) -> Result<Response, ContractError> {
        let token = deps.api.addr_validate(&token_address)?;

        let depositor_address = &info.sender;
        let contract_address = &env.contract.address;

        let query_msg = to_binary(&Cw20QueryMsg::Allowance {
            owner: depositor_address.into(),
            spender: contract_address.into(),
        })?;

        let allowance: Uint128 = deps.querier.query_wasm_smart(token.clone(), &query_msg)?;

        //check allowance
        if allowance < amount {
            //TODO: create specific error
            return Err(ContractError::InsufficientTokenAllowance{});
        }

        let transfer_token_msg = to_binary(&Cw20ExecuteMsg::TransferFrom {
            owner: depositor_address.into(),
            recipient: contract_address.into(),
            amount,
        })?;

        let execute_msg = WasmMsg::Execute {
            contract_addr: contract_address.into(),
            msg: transfer_token_msg,
            funds: vec![],
        };

        //transfer sub msg
        let transfer_sub_msg = SubMsg::reply_always(execute_msg, SUCCESS_REPLY_MSG);

        //create xcall rlp encode data
        let xcall_data = Deposit {
            token_address: token_address.clone(),
            from: depositor_address.to_string(),
            to: to.clone(),
            amount: Uint128::u128(&amount),
            data,
        };

       
        
        let source_xcall = SOURCE_XCALL.load(deps.storage)?;
        //create xcall msg for dispatching  send call
        let xcall_message = XCallMsg::SendCallMessage {
            to: to.clone(),
            data: xcall_data.rlp_bytes().to_vec(),
            //TODO: add the rollback with deposit revert information
            rollback: Some(
                DepositRevert {
                    token_address,
                    account: depositor_address.to_string(),
                    amount: Uint128::u128(&amount),
                }
                .rlp_bytes()
                .to_vec(),
            ),
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

        let resp = Response::new().add_submessages(vec![transfer_sub_msg, xcall_sub_msg]).add_event(event);

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
        let xcall_addr = deps.api.addr_validate(&xcall)?;

        if info.sender != xcall_addr {
            return Err(ContractError::OnlyXcallService);
        }

        let (_, decoded_struct) = decode_encoded_bytes(&data)?;

        match decoded_struct {
            DecodedStruct::DepositRevert(data) => {
                //TODO: _from should be with network address of xcall in archway
                let network_address = NetworkAddress::new("0x44.arch", &from);
                let checked_from = NetworkAddress::from_str(&network_address.to_string())?;
                let x_network = X_NETWORK_ADDRESS.load(deps.storage)?;

                if checked_from.to_string() != x_network.to_string() {
                    return  Err(ContractError::OnlyXcallService);
                } 

                
                let token_address = data.token_address;
                let account = data.account;
                let amount = Uint128::from(data.amount);
                transfer_tokens(deps, account, token_address, amount)?;
            }

            DecodedStruct::WithdrawTo(data_struct) => {
                //TODO: Check if _from is ICON Asset manager contract
                let icon_am = ICON_ASSET_MANAGER.load(deps.storage)?;
                if from != icon_am.to_string() {
                    return  Err(ContractError::OnlyIconAssetManager{});
                }
                let token_address = data_struct.token_address;
                let account = data_struct.user_address;
                let amount = Uint128::from(data_struct.amount);

                transfer_tokens(deps, account, token_address, amount)?;
            } 

            DecodedStruct::WithdrawRequest(data_struct) => {
                let network_address = NetworkAddress::new("0x44.arch", &from);
                let checked_from = NetworkAddress::from_str(&network_address.to_string())?;
                let x_network = X_NETWORK_ADDRESS.load(deps.storage)?;

                if checked_from.to_string() != x_network.to_string() {
                    return  Err(ContractError::OnlyXcallService);
                } 
                
                let token_address = data_struct.token_address;
                let recipient = data_struct.to;
                let amount = Uint128::from(data_struct.amount);

                transfer_tokens(deps, recipient, token_address, amount)?;
            } 
            //unknown received data type will be handled at decoding()
        }

        Ok(Response::default()) 
    }

    //internal function to transfer tokens from contract to account
    pub fn transfer_tokens(
        _deps: DepsMut,
        account: String,
        token_address: String,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        let account = Addr::unchecked(account);
        let token_address = Addr::unchecked(token_address);

        let transfer_msg = &Cw20ExecuteMsg::Transfer {
            recipient: account.to_string(),
            amount,
        };

        let execute_msg = WasmMsg::Execute {
            contract_addr: token_address.to_string(),
            msg: to_binary(transfer_msg)?,
            funds: vec![],
        };

        let sub_msg = SubMsg::reply_always(execute_msg, SUCCESS_REPLY_MSG);
        Ok(Response::new().add_submessage(sub_msg))
    }
}



#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
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
    use crate::contract::exec::configure_network;

    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier},
        Api, ContractResult, MemoryStorage, OwnedDeps, SystemResult, Uint128, WasmQuery,
    };
    use rlp::Encodable;

    use cw_common::xcall_data_types::DepositRevert;
    use cw_common::{asset_manager_msg::InstantiateMsg, xcall_data_types::WithdrawRequest};

    use super::*;

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

        let msg = InstantiateMsg {
            cw20_whitelist: vec!["token1".to_owned(), "token2".to_owned()],
        };

        let instantiated_resp = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        //to pretend us as xcall contract during handle call execution testing
        let xcall = "user";

        let configure_msg = ExecuteMsg::ConfigureXcall {
            source_xcall: xcall.to_owned(),
            destination_asset_manager: "0x38.icon/cxc2d01de5013778d71d99f985e4e2ff3a9b48a66c"
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
                        to_binary(
                            &"0x44.arch/archway1q28lhwcjeq6wak6aypcgtpv7jd5d7skm8xszvg".to_owned(),
                        )
                        .unwrap(),
                    ))
                } else {
                    SystemResult::Ok(ContractResult::Ok(to_binary(&Uint128::new(1000)).unwrap()))
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

        let token1_validated = VALID_TOKENS
            .load(
                deps.as_ref().storage,
                &deps.api.addr_validate("token1").unwrap(),
            )
            .unwrap();
        assert!(token1_validated);

        let token2_validated = VALID_TOKENS
            .load(
                deps.as_ref().storage,
                &deps.api.addr_validate("token2").unwrap(),
            )
            .unwrap();
        assert!(token2_validated);
    }

    // #[test]
    fn test_deposit_for_sufficient_allowance() -> (
        OwnedDeps<MemoryStorage, MockApi, MockQuerier>,
        Env,
        MessageInfo,
    ) {
        let (mut deps, env, info, _) = test_setup();

        let destination_asset_manager = ICON_LOANS_ADDRESS.load(deps.as_ref().storage).unwrap();
        assert_eq!(
            destination_asset_manager,
            "0x38.icon/cxc2d01de5013778d71d99f985e4e2ff3a9b48a66c".to_string()
        );

        // Test Deposit message
        let msg = ExecuteMsg::Deposit {
            token_address: "token1".to_string(),
            amount: Uint128::new(100),
        };

        let result = execute(deps.as_mut(), env.clone(), info.clone(), msg);

        //alternative: directly assert the unwrapped value
        //  OR,

        match result {
            Ok(response) => {
                // Verify the response contains the expected sub messages
                assert_eq!(response.messages.len(), 2);

                let depositor = Addr::unchecked("user");
                let token = Addr::unchecked("token1");

                //asserting state change
                let deposit = DEPOSITS.load(&deps.storage, (&depositor, &token)).unwrap();
                assert_eq!(deposit, Uint128::new(100));
            }
            Err(error) => {
                panic!("Unexpected error occurred: {:?}", error);
            }
        }

        (deps, env, info)
    }

    #[test]
    fn test_deposit_for_insufficient_allowance() {
        let (mut deps, env, info, _) = test_setup();

        let destination_asset_manager = ICON_LOANS_ADDRESS.load(deps.as_ref().storage).unwrap();
        assert_eq!(
            destination_asset_manager,
            "0x38.icon/cxc2d01de5013778d71d99f985e4e2ff3a9b48a66c".to_string()
        );

        // Test Deposit message
        let msg = ExecuteMsg::Deposit {
            token_address: "token1".to_string(),
            amount: Uint128::new(1500),
        };

        let result = execute(deps.as_mut(), env, info, msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_withdraw_request() {
        let (mut deps, env, info) = test_deposit_for_sufficient_allowance();

        let msg = ExecuteMsg::WithdrawRequest {
            token_address: "token1".to_string(),
            amount: Uint128::new(50),
        };

        let result = execute(deps.as_mut(), env, info, msg);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_xcall() {
        //"user" : type(addr) is set in the contract as xcall contract for testing
        //reason: executor is "user" on testing
        let (mut deps, env, info) = test_deposit_for_sufficient_allowance();

        let xcall = info.sender.to_string();
        //create deposit revert(expected)  xcall msg deps
        let x_deposit_revert = DepositRevert {
            token_address: "token1".to_string(),
            account: "user".to_string(),
            amount: 100,
        };

        //create valid handle_call_message
        let msg = ExecuteMsg::HandleCallMessage {
            from: xcall.clone(),
            data: x_deposit_revert.rlp_bytes().to_vec(),
        };

        let result = execute(deps.as_mut(), env.clone(), info.clone(), msg);

        //check for valid xcall expected msg data
        assert!(result.is_ok());

        let current_balance = DEPOSITS
            .load(
                &deps.storage,
                (&Addr::unchecked(xcall.clone()), &Addr::unchecked("token1")),
            )
            .unwrap();
        //confirm state change for successful deposit revert
        assert!(current_balance.is_zero());

        let x_msg = WithdrawRequest {
            token_address: "token1".to_owned(),
            from: "account1".to_string(),
            amount: 1280,
        };

        let unknown_msg = ExecuteMsg::HandleCallMessage {
            from: xcall,
            data: x_msg.rlp_bytes().to_vec(),
        };

        //check for error due to unknown xcall handle data
        let result = execute(deps.as_mut(), env, info, unknown_msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_configure_network() {
        // Prepare the test data
        let mut deps = mock_dependencies();
        let owner = "owner";
        let source_xcall = "source_xcall";
        let destination_asset_manager = "destination_asset_manager";

        // Set the owner
        OWNER
            .save(&mut deps.storage, &Addr::unchecked(owner))
            .unwrap();

        // Prepare the message info with the owner as the sender
        let info = mock_info(&owner, &[]);

        // Execute the function
        let res = configure_network(
            deps.as_mut(),
            info.clone(),
            source_xcall.to_string(),
            destination_asset_manager.to_string(),
        );

        // Check the response
        assert!(res.is_ok());
        let response: Response = res.unwrap();
        assert_eq!(response, Response::default());

        // Verify the saved values
        let saved_source_xcall: String = SOURCE_XCALL.load(deps.as_ref().storage).unwrap();
        let saved_destination_asset_manager: String =
            ICON_LOANS_ADDRESS.load(deps.as_ref().storage).unwrap();

        assert_eq!(saved_source_xcall, source_xcall);
        assert_eq!(saved_destination_asset_manager, destination_asset_manager);

        // Verify that only the owner can configure the network
        let other_sender = "other_sender";
        let other_info = mock_info(&other_sender, &[]);
        let res = configure_network(
            deps.as_mut(),
            other_info,
            source_xcall.to_string(),
            destination_asset_manager.to_string(),
        );

        assert!(res.is_err());
        let err = res.unwrap_err();
        assert_eq!(err, ContractError::OnlyOwner);
    }
}
