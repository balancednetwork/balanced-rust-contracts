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
    ExecuteMsg::WithdrawRequest { token_address, amount } => exec::withdraw_request(),

   } 
   
}


#[allow(dead_code)]
 mod exec {
use cw_utils::parse_execute_response_data;




use super::*;

    pub fn deposit_cw20_tokens(deps:DepsMut,info: MessageInfo,env : Env,token_address: String,token_amount: Uint128) -> Result<Response,ContractError> {
     let token = deps.api.addr_validate(&token_address)?;

     //VALIDATE TOKEN 
     if !helpers::is_valid_token(&deps,&token) {
        return Err(ContractError::InvalidToken { address: token });
     }
     
     let depositor_address = &info.sender;
     let contract_address = &env.contract.address;
     
    let query_msg = to_binary(&Cw20QueryMsg::Allowance { owner: depositor_address.into(), spender: contract_address.into() })?;

     let allowance: Uint128 = deps.querier.query_wasm_smart(token.clone(), &query_msg)?;

     //check allowance
     if allowance < token_amount {
        return  Err(StdError::generic_err("CW20: Insufficient Allowance")).map_err(ContractError::Std);
     }
    
    //execute transfer call on  external cw20 contract
    let emsg = &Cw20ExecuteMsg::TransferFrom { 
        owner: depositor_address.into(), 
        recipient: contract_address.into(), 
        amount: token_amount,
    };
    let emsg_binary = to_binary(emsg).unwrap();
    let execute_msg = WasmMsg::Execute { contract_addr: contract_address.into(), msg: emsg_binary, funds: vec![] };


    let  resp = Response::new().add_submessage(SubMsg::reply_on_success(execute_msg, DEPOSIT_MSG_ID)).add_attribute("action", "Recieved Deposits");
   


    // //update deposit state of the depositor
    // let current_deposits =DEPOSITS.load(deps.storage, (depositor_address,&token))?; 
    // DEPOSITS.save(deps.storage,(&depositor_address,&token),&(current_deposits + token_amount))?;
    

    Ok(resp)


     
    }

  

    pub fn withdraw_request() -> Result<Response, ContractError> {
        unimplemented!()
    }





    pub fn deposit_reply_handler(deps: DepsMut,msg:SubMsgResult) -> Result<Response, ContractError>{
        //extract 'Response' from 'SubMsgResult
        let resp = match msg.into_result() {
            Ok(resp) => resp,
            Err(err) => return Err(StdError::generic_err(err))?,
        };
        
        //extract 'data' field from 'resp'
        let data = resp.data.ok_or_else(|| StdError::generic_err("No transfer response data"))?;

         let resp = parse_execute_response_data(&data).map(|err| ContractError::ErrorInParsing {  });
         
         //update state
         //send xcall msg
        Ok(Response::new())
    }


    mod helpers {
        use super::*;


        pub fn is_valid_token(deps: &DepsMut,token: &Addr) -> bool {
           VALID_TOKENS.load(deps.storage, token).unwrap()
        }


        pub fn send_xcall_msg() -> StdResult<Response> {
            unimplemented!()
        }

    
    }
}





#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}




pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        DEPOSIT_MSG_ID => exec::deposit_reply_handler(deps, msg.result),
        _ => Err(StdError::generic_err("unknown reply id"))?,
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::Empty;
    use cw_multi_test::{App, Contract, ContractWrapper, Executor};

    // fn cw20() -> Box<dyn Contract<Empty>> {
    //    let contract = ContractWrapper::new(Cw20ExecuteMsg,Cw20QueryMsg);
    // }
}
