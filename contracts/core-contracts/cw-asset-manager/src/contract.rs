use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, Event, MessageInfo, QueryRequest, Reply, Response,
    StdError, StdResult, SubMsg, SubMsgResult, Uint128, WasmMsg, WasmQuery,
};

use crate::constants::SUCCESS_REPLY_MSG;
use crate::error::ContractError;
use crate::helpers::{decode_encoded_bytes, DecodedStruct};
use crate::state::*;

use cw_common::asset_manager_msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use cw_common::xcall_data_types::Deposit;
use cw_common::xcall_msg::XCallMsg;

use cw2::set_contract_version;
use cw20::{Cw20ExecuteMsg, Cw20QueryMsg};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-asset-manager";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    OWNER.save(deps.storage, &info.sender)?;

    for addr in msg.cw20_whitelist.iter() {
        let token = &deps.api.addr_validate(addr)?;
        VALID_TOKENS.save(deps.storage, token, &true)?;
    }

    Ok(Response::new()
        .add_attribute("action", "instantiated")
        .add_attribute("deployed address", env.contract.address)
        .add_attribute("deployer", info.sender))
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
            destination_contract,
        } => exec::configure_network(deps, info, source_xcall, destination_contract),
        ExecuteMsg::HandleCallMessage { from, data } => {
            exec::handle_xcallmsg(deps, env, info, from, data)
        }
        ExecuteMsg::Deposit {
            token_address,
            amount,
        } => exec::deposit_cw20_tokens(deps, info, env, token_address, amount),
        ExecuteMsg::WithdrawRequest {
            token_address,
            amount,
        } => exec::withdraw_request(deps, info, token_address, amount),
    }
}

#[allow(dead_code)]
mod exec {

    use cw_common::xcall_data_types::WithdrawRequest;
    use rlp::Encodable;

    use super::*;

    pub fn configure_network(
        deps: DepsMut,
        _info: MessageInfo,
        source_xcall: String,
        destination_contract: String,
    ) -> Result<Response, ContractError> {
        // let query_msg = XCallQuery::GetNetworkAddress { };

        // //get the network address of the destination xcall contract
        // let query = QueryRequest::Wasm(WasmQuery::Smart {
        //     contract_addr: source_xcall.to_string(),
        //      msg: to_binary(&query_msg)?
        //     });

        // let destn_xcall_btp_address: String = deps.querier.query(&query)?;

        // if destn_xcall_btp_address.is_empty() {
        //     return Err(ContractError::AddressNotFound)
        // }

        SOURCE_XCALL.save(deps.storage, &source_xcall)?;
        DEST_CONTRACT_BTP_ADDR.save(deps.storage, &destination_contract)?;
        Ok(Response::default())
    }

    pub fn deposit_cw20_tokens(
        deps: DepsMut,
        info: MessageInfo,
        env: Env,
        token_address: String,
        token_amount: Uint128,
    ) -> Result<Response, ContractError> {
        let token = deps.api.addr_validate(&token_address)?;

        //VALIDATE TOKEN
        if !VALID_TOKENS.load(deps.storage, &token).unwrap() {
            return Err(ContractError::InvalidToken { address: token });
        }

        let depositor_address = &info.sender;
        let contract_address = &env.contract.address;

        let query_msg = to_binary(&Cw20QueryMsg::Allowance {
            owner: depositor_address.into(),
            spender: contract_address.into(),
        })?;

        let allowance: Uint128 = deps.querier.query_wasm_smart(token.clone(), &query_msg)?;

        //check allowance
        if allowance < token_amount {
            return Err(StdError::generic_err("CW20: Insufficient Allowance").into());
        }

        //execute transfer call on  external cw20 contract
        let emsg = &Cw20ExecuteMsg::TransferFrom {
            owner: depositor_address.into(),
            recipient: contract_address.into(),
            amount: token_amount,
        };

        let emsg_binary = to_binary(emsg)?;
        let execute_msg = WasmMsg::Execute {
            contract_addr: contract_address.into(),
            msg: emsg_binary,
            funds: vec![],
        };

        //transfer submsg
        let transfer_submsg = SubMsg::reply_always(execute_msg, SUCCESS_REPLY_MSG);

        //create xcall rlp encode data
        let xcall_data = Deposit {
            token_address: token_address.clone(),
            from: info.sender.to_string(),
            to: env.contract.address.to_string(),
            amount: Uint128::u128(&token_amount),
        };

        let to_addr = DEST_CONTRACT_BTP_ADDR.load(deps.storage)?;
        let source_xcall = SOURCE_XCALL.load(deps.storage)?;
        //create xcall msg for dispatching  sendcall
        let xcall_messag = XCallMsg::SendCallMessage {
            to: to_addr,
            data: xcall_data.rlp_bytes().to_vec(),
            rollback: None,
        };

        let xcall_msg = WasmMsg::Execute {
            contract_addr: source_xcall,
            msg: to_binary(&xcall_messag)?,
            funds: vec![],
        };

        let xcall_submsg = SubMsg::reply_always(xcall_msg, SUCCESS_REPLY_MSG);

        // Update state
        let current_balance = DEPOSITS
            .may_load(deps.storage, (&depositor_address, &token))?
            .unwrap_or_else(|| Uint128::zero());

        let updated_balance = current_balance + token_amount;
        DEPOSITS.save(deps.storage, (&depositor_address, &token), &updated_balance)?;

        let resp = Response::new().add_submessages(vec![transfer_submsg, xcall_submsg]);

        Ok(resp)
    }

    pub fn withdraw_request(
        deps: DepsMut,
        info: MessageInfo,
        token_address: String,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        //check withdrawer's current token balance
        let token = deps.api.addr_validate(&token_address)?;
        if DEPOSITS.has(deps.storage, (&info.sender, &token)) {
            let current_balance = DEPOSITS.load(deps.storage, (&info.sender, &token))?;
            if current_balance.is_zero() {
                return Err(ContractError::InsufficentTokenBalance {});
            }
        }

        let withdrawer = &info.sender;
        let call_data = WithdrawRequest {
            token_address,
            from: withdrawer.into(),
            amount: Uint128::u128(&amount),
        };

        let to_addr = DEST_CONTRACT_BTP_ADDR.load(deps.storage)?;
        let source_xcall = SOURCE_XCALL.load(deps.storage)?;
        //create xcall msg for dispatching  sendcall
        let xcall_messag = XCallMsg::SendCallMessage {
            to: to_addr,
            data: call_data.rlp_bytes().to_vec(),
            rollback: None,
        };

        let xcall_msg = WasmMsg::Execute {
            contract_addr: source_xcall,
            msg: to_binary(&xcall_messag)?,
            funds: vec![],
        };

        let xcall_submsg = SubMsg::reply_always(xcall_msg, SUCCESS_REPLY_MSG);

        let attributes = vec![
            ("Token", token.to_string()),
            ("From", withdrawer.to_string()),
            ("Amount", amount.to_string()),
        ];

        let event = Event::new("Withdraw").add_attributes(attributes);

        let resp = Response::new()
            .add_submessage(xcall_submsg)
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

    pub fn handle_xcallmsg(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        _from: String,
        data: Vec<u8>,
    ) -> Result<Response, ContractError> {
        let xcall = SOURCE_XCALL.load(deps.storage)?;

        if info.sender.to_string() != xcall {
            return Err(ContractError::OnlyXcallService);
        }

        let (_, decoded_struct) = decode_encoded_bytes(&data)?;

        match decoded_struct {
            DecodedStruct::DepositRevert(data) => {
                let token_address = data.token_address;
                let account = data.account;
                let amount = Uint128::from(data.amount);
                transfer_tokens(deps, account, token_address, amount)?;
            }

            DecodedStruct::WithdrawTo(data_struct) => {
                let token_address = data_struct.token_address;
                let account = data_struct.user_address;
                let amount = Uint128::from(data_struct.amount);

                transfer_tokens(deps, account, token_address, amount)?;
            } //unknown recieved data type will be handled at decoding()
        }

        Ok(Response::default())
    }

    //helper function to transfer tokens from contract to account
    pub fn transfer_tokens(
        deps: DepsMut,
        account: String,
        token_address: String,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        let account = Addr::unchecked(account);
        let token_address = Addr::unchecked(token_address);

        let current_balance = DEPOSITS.load(deps.storage, (&account, &token_address))?;

        if amount > current_balance {
            return Err(ContractError::InsufficentTokenBalance {});
        }

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

        // Update state
        let current_balance = DEPOSITS.load(deps.storage, (&account, &token_address))?;
        DEPOSITS.save(
            deps.storage,
            (&account, &token_address),
            &(current_balance - amount),
        )?;
        Ok(Response::new().add_submessage(sub_msg))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        SUCCESS_REPLY_MSG => exec::reply_handler(msg.result),
        _ => Err(StdError::generic_err("unknown reply id"))?,
    }
}

#[cfg(test)]

mod tests {

    use super::*;

    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier},
        Api, ContractResult, MemoryStorage, OwnedDeps, SystemResult, Uint128,
    };
    use cw_common::xcall_data_types::DepositRevert;
    use cw_common::{asset_manager_msg::InstantiateMsg, xcall_data_types::WithdrawRequest};
    use rlp::Encodable;

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

        //to pretend us as xcall contract during handlecall execution testing
        let xcall = "user";

        let configure_msg = ExecuteMsg::ConfigureXcall {
            source_xcall: xcall.to_owned(),
            destination_contract: "btp://0x38.icon/abcdefghijklmnop".to_owned(),
        };

        // mocking response for external query i.e. allowance
        deps.querier.update_wasm(|r: &WasmQuery| match r {
            WasmQuery::Smart {
                contract_addr,
                msg: _,
            } => {
                if contract_addr == &xcall.to_owned() {
                    SystemResult::Ok(ContractResult::Ok(
                        to_binary(&"btp://0x38.icon/abcdefghijklmnop".to_owned()).unwrap(),
                    ))
                } else {
                    SystemResult::Ok(ContractResult::Ok(to_binary(&Uint128::new(1000)).unwrap()))
                }
            }
            _ => todo!(),
        });

        execute(deps.as_mut(), env.clone(), info.clone(), configure_msg).unwrap();

        (deps, env.clone(), info.clone(), instantiated_resp)
    }

    #[test]
    fn test_instantiate() {
        let (deps, _, info, res) = test_setup();

        //check proper instantiation
        assert_eq!(res.attributes.len(), 3);
        assert_eq!(res.attributes[0], ("action", "instantiated"));

        let owner = OWNER.load(&deps.storage).unwrap();
        assert_eq!(owner, info.sender);

        let token1_validated = VALID_TOKENS
            .load(
                deps.as_ref().storage,
                &deps.api.addr_validate("token1").unwrap(),
            )
            .unwrap();
        assert_eq!(token1_validated, true);

        let token2_validated = VALID_TOKENS
            .load(
                deps.as_ref().storage,
                &deps.api.addr_validate("token2").unwrap(),
            )
            .unwrap();
        assert_eq!(token2_validated, true);
    }

    // #[test]
    fn test_deposit_for_sufficient_allowance() -> (
        OwnedDeps<MemoryStorage, MockApi, MockQuerier>,
        Env,
        MessageInfo,
    ) {
        let (mut deps, env, info, _) = test_setup();

        let destination_contract = DEST_CONTRACT_BTP_ADDR.load(deps.as_ref().storage).unwrap();
        assert_eq!(
            destination_contract,
            "btp://0x38.icon/abcdefghijklmnop".to_string()
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
                // Verify the response contains the expected submessages
                assert_eq!(response.messages.len(), 2);

                let depositor = Addr::unchecked("user");
                let token = Addr::unchecked("token1");

                //asserting state change
                let deposit = DEPOSITS.load(&deps.storage, (&depositor, &token)).unwrap();
                assert_eq!(deposit, Uint128::new(100));
            }
            Err(error) => {
                panic!("Unexpected error occured: {:?}", error);
            }
        }

        (deps, env, info.clone())
    }

    #[test]
    fn test_deposit_for_insufficient_allowance() {
        let (mut deps, env, info, _) = test_setup();

        let destination_contract = DEST_CONTRACT_BTP_ADDR.load(deps.as_ref().storage).unwrap();
        assert_eq!(
            destination_contract,
            "btp://0x38.icon/abcdefghijklmnop".to_string()
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
        //create deposit revert(expected)  xcall msgdeps
        let x_deposit_revert = DepositRevert {
            token_address: "token1".to_string(),
            account: "user".to_string(),
            amount: 100,
        };

        let encoded_xdata = x_deposit_revert.rlp_bytes().to_vec();

        //create valid handlecall message
        let msg = ExecuteMsg::HandleCallMessage {
            from: xcall.clone(),
            data: encoded_xdata,
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

        let unkown_msg = ExecuteMsg::HandleCallMessage {
            from: xcall.to_owned(),
            data: x_msg.rlp_bytes().to_vec(),
        };

        //check for error due to unknown xcall handle data
        let result = execute(deps.as_mut(), env, info, unkown_msg);
        assert!(result.is_err());
    }
}
