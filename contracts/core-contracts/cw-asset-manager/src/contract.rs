#[cfg(not(feature = "library"))]


use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Uint128,Deps, DepsMut, Env, MessageInfo, Response, StdResult,Addr,WasmMsg,SubMsg,SubMsgResult,Reply,StdError,to_binary};
use cw2::set_contract_version;

use crate::constants::{DEPOSIT_MSG_ID,WITHDRAW_MSG_ID};
use crate::error::ContractError;
use crate::msg::{InstantiateMsg, QueryMsg};
use crate::state::{DEPOSITS,OWNER,VALID_TOKENS};


use cw_common::asset_manager_msg::*;

use cw20::{Cw20ExecuteMsg,Cw20QueryMsg};



// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-asset-manager";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage,CONTRACT_NAME,CONTRACT_VERSION)?;
     OWNER.save(deps.storage,&info.sender)?;
   
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
    ExecuteMsg::Deposit { token_address, amount } => exec::deposit_cw20_tokens(deps,info,env,token_address,amount),
    ExecuteMsg::WithdrawRequest { token_address, amount } => exec::withdraw_request(deps,info,env,token_address,amount),

   } 
   
}


#[allow(dead_code)]
 mod exec {
    
use std::str::FromStr;

use super::*;

    pub fn deposit_cw20_tokens(deps:DepsMut,info: MessageInfo,env : Env,token_address: String,token_amount: Uint128) -> Result<Response,ContractError> {
     
      
     let token = deps.api.addr_validate(&token_address)?;


     //VALIDATE TOKEN 
     if !VALID_TOKENS.load(deps.storage, &token).unwrap() {
        return Err(ContractError::InvalidToken { address: token });
     }
     
     let depositor_address = &info.sender;
     let contract_address = &env.contract.address;
     
    let query_msg = to_binary(&Cw20QueryMsg::Allowance { owner: depositor_address.into(), spender: contract_address.into() })?;

     let allowance: Uint128 = deps.querier.query_wasm_smart(token.clone(), &query_msg)?;

     //check allowance
     if allowance < token_amount {
        return  Err(StdError::generic_err("CW20: Insufficient Allowance"))?;
     }
    
    //execute transfer call on  external cw20 contract
    let emsg = &Cw20ExecuteMsg::TransferFrom { 
        owner: depositor_address.into(), 
        recipient: contract_address.into(), 
        amount: token_amount,
    };
    let emsg_binary = to_binary(emsg).unwrap();
    let execute_msg = WasmMsg::Execute { contract_addr: contract_address.into(), msg: emsg_binary, funds: vec![] };
    
    //transfer submsg
    let transfer_submsg = SubMsg::reply_on_success(execute_msg, DEPOSIT_MSG_ID);

    //add xcall submsg
    let xcall_submsg;


    let  resp = Response::new().add_submessages(vec![transfer_submsg,xcall_submsg]).add_attribute("action", "Recieved Deposits");

   todo!()
     
    }

  

    pub fn withdraw_request(deps:DepsMut,info: MessageInfo,env : Env,token_address: String,token_amount: Uint128) -> Result<Response, ContractError> {
       
        let withdrawer = &info.sender;
        let token = &deps.api.addr_validate(&token_address)?;

        let current_balance = DEPOSITS.load(deps.storage, (withdrawer,token))?;

        if current_balance < token_amount {
            return  Err(StdError::generic_err("CW20: Withdraw Amount Exceeds Your Balance"))?;
        }

        let msg = &Cw20ExecuteMsg::Transfer { 
            recipient: withdrawer.into(),
             amount: token_amount,
             };
        let execute_msg = WasmMsg::Execute { 
            contract_addr: token_address.into(),
             msg: to_binary(msg).unwrap(),
              funds: vec![],
             };

        let resp = Response::new().add_submessage(SubMsg::reply_on_success(execute_msg, WITHDRAW_MSG_ID)).add_attribute("action", "withdraw");
        
        Ok(resp)
    }




    pub fn deposit_reply_handler(deps: DepsMut, msg: SubMsgResult, env: Env) -> Result<Response, ContractError> {
        let mut attribute_vec: Vec<String> = Vec::new();
        let result = msg.into_result();
        match result {
            Ok(resp) => {
                if resp.events.is_empty() {
                    return Ok(Response::default());
                }
    
                for event in resp.events {
                    for attr in event.attributes {
                        let key = attr.key;
                        let value = attr.value;
                        attribute_vec.push(value);
                    }
                }
    
                let depositor = &Addr::unchecked(attribute_vec[1]);
                let amount = Uint128::from_str(&attribute_vec[4]);
                let token_address = env.contract.address;
    
                // Update state
                let current_deposits = DEPOSITS.load(deps.storage, (depositor, &token_address))?;
                DEPOSITS.save(deps.storage, (&depositor, &token_address), &(current_deposits + amount.unwrap()))?;

    
                Ok(Response::default())
            },
            Err(err) => Err(ContractError::TokenTransferFailure{reason: err}),
        }
    }
    


    

    pub fn withdraw_reply_handler(deps: DepsMut,msg:SubMsgResult) -> Result<Response, ContractError>{
        //extract 'Response' from 'SubMsgResult
        let result = msg.into_result();
        match result {
            Ok(resp) => {
            

            },
            Err(err) => return Err(StdError::generic_err(err))?,
        };
        
        todo!()
       
    }


}





#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}



#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        DEPOSIT_MSG_ID => exec::deposit_reply_handler(deps, msg.result,env),
        WITHDRAW_MSG_ID => exec::deposit_reply_handler(deps, msg.result,env),
        _ => Err(StdError::generic_err("unknown reply id"))?,
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{attr, coin, coins, CosmosMsg, StdError, Uint128};
    use super::*;


    
 
}
