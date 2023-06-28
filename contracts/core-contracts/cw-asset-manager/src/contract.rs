#[cfg(not(feature = "library"))]


use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Uint128,Deps, DepsMut, Env, MessageInfo, Response, StdResult,Addr,WasmMsg,SubMsg,SubMsgResult,Reply,StdError,to_binary, QueryRequest,WasmQuery};


use crate::constants::SUCCESS_REPLY_MSG;
use crate::error::ContractError;
use crate::helpers::{decode_encoded_bytes,DecodedStruct};
use crate::state::*;


use cw_common::asset_manager_msg::{InstantiateMsg,ExecuteMsg,QueryMsg};
use cw_common::xcall_msg::{XCallMsg,XCallQuery};
use cw_common::xcall_data_types::{Deposit,DepositRevert};

use cw20::{Cw20ExecuteMsg,Cw20QueryMsg};
use cw2::set_contract_version;




// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-asset-manager";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage,CONTRACT_NAME,CONTRACT_VERSION)?;
    
    OWNER.save(deps.storage, &info.sender)?;

    for addr in msg.cw20_whitelist.iter() {
        let token = &deps.api.addr_validate(addr)?;
        VALID_TOKENS.save(deps.storage, token, &true)?;
       
    }
    
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
    ExecuteMsg::ConfigureXcall { source_xcall, destination_contract } => exec::configure_network(deps, source_xcall,destination_contract),
    ExecuteMsg::HandleCallMessage { from, data } => exec::handle_xcallmsg(deps,env,info,from,data),
    ExecuteMsg::Deposit { token_address, amount } => exec::deposit_cw20_tokens(deps,info,env,token_address,amount),
    ExecuteMsg::WithdrawRequest { token_address, amount } => exec::withdraw_request(deps,info,env,token_address,amount),

   } 
   
}


#[allow(dead_code)]
 mod exec {
    
use cosmwasm_std::Event;
use cw_common::xcall_data_types::WithdrawRequest;
use rlp::Encodable;

use super::*;


pub fn configure_network(deps: DepsMut,source_xcall:String,destination_contract: String) -> Result<Response,ContractError> {
    let source_xcall = &source_xcall;
    let query_msg = XCallQuery::GetNetworkAddress { };

    //get the network address of the destination xcall contract
    let query = QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: source_xcall.to_string(),
         msg: to_binary(&query_msg)?
        });

    
    let destn_xcall_btp_address: String = deps.querier.query(&query)?;
    if destn_xcall_btp_address.is_empty() {
        return Err(ContractError::AddressNotFound)
    }

    DEST_CONTRACT_BTP_ADDR.save(deps.storage, &destination_contract)?;
    Ok(Response::default())
}





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
    let transfer_submsg = SubMsg::reply_always(execute_msg, SUCCESS_REPLY_MSG);

  

    //create xcall rlp encode data 
    let xcall_data = Deposit {
        token_address : token_address.clone(),
        from: info.sender.to_string(),
        to: env.contract.address.to_string(),
        amount: Uint128::u128(&token_amount),
         
   };


   let to_addr = DEST_CONTRACT_BTP_ADDR.load(deps.storage)?;
   let source_xcall =SOURCE_XCALL.load(deps.storage)?;
   //create xcall msg for dispatching  sendcall
   let xcall_messag = XCallMsg::SendCallMessage { 
    to: to_addr,
     data: xcall_data.rlp_bytes().to_vec(), 
    rollback: None,
 };

   let xcall_msg = WasmMsg::Execute { 
    contract_addr: source_xcall.to_string(),
     msg: to_binary(&xcall_messag)?,
      funds: vec![],
     };

     let xcall_submsg = SubMsg::reply_always(xcall_msg, SUCCESS_REPLY_MSG);

  

        // Update state
        let current_balance = DEPOSITS.load(deps.storage, (depositor_address, &token))?;
        DEPOSITS.save(deps.storage, (&depositor_address, &token), &(current_balance + token_amount))?; 


     
    let  resp = Response::new().add_submessages(vec![transfer_submsg,xcall_submsg]);

    Ok(resp)
     
    }

  

    pub fn withdraw_request(deps:DepsMut,info: MessageInfo,env : Env,token_address: String,amount: Uint128) -> Result<Response, ContractError> {
       
        let withdrawer = &info.sender;
        let token = deps.api.addr_validate(&token_address)?;

        let call_data = WithdrawRequest{
            token_address : token_address.clone(),
            from: withdrawer.into(),
            amount: Uint128::u128(&amount)
        };

        //will be changed later
        let rollback_data = DepositRevert {
            token_address:token_address,
            account: withdrawer.into(),
            amount: Uint128::u128(&amount),
           };
        

        let to_addr = DEST_CONTRACT_BTP_ADDR.load(deps.storage)?;
        let source_xcall =SOURCE_XCALL.load(deps.storage)?;
        //create xcall msg for dispatching  sendcall
        let xcall_messag = XCallMsg::SendCallMessage { 
         to: to_addr,
          data: call_data.rlp_bytes().to_vec(), 
         rollback: Some(rollback_data.rlp_bytes().to_vec()),
      };
        


      let xcall_msg = WasmMsg::Execute { 
        contract_addr: source_xcall.to_string(),
         msg: to_binary(&xcall_messag)?,
          funds: vec![],
         };
    
         let xcall_submsg = SubMsg::reply_always(xcall_msg, SUCCESS_REPLY_MSG);
         let resp = Response::new().add_submessage(xcall_submsg);
         transfer_tokens(deps, withdrawer.into(), token.into(), amount)?;

        //  let attributes = vec![
        //     ("Token", token_address.to_string()),
        //     ("From", withdrawer.to_string()),
        //     ("Amount", amount.to_string()),
        // ];
    
        // let event = Event::new("Withdraw").add_attributes(attributes);  


         Ok(resp)
    }






    pub fn reply_handler(deps: DepsMut,msg:SubMsgResult) -> Result<Response, ContractError>{
        let result = msg.into_result();
        match result {
            Ok(_) => {
            Ok(Response::default())
            },
            Err(err) => return Err(StdError::generic_err(err))?,
        }      
    }



    pub fn handle_xcallmsg(        
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        from: String,
        data: Vec<u8>
    ) -> Result<Response,ContractError> {
         let xcall = SOURCE_XCALL.load(deps.storage)?;

         if info.sender != xcall {
            return Err(ContractError::OnlyXcallService)
         }
         
        let (_,decoded_struct) = decode_encoded_bytes(&data)?;

        match decoded_struct {
            DecodedStruct::DepositRevert(data) => {
                let token_address = data.token_address;
                let account = data.account;
                let amount = Uint128::new(data.amount);
                transfer_tokens(deps, account, token_address, amount)?;
            },

            DecodedStruct::WithdrawTo(data_struct) => {
                let token_address= data_struct.token_address;
                let account = data_struct.user_address;
                let amount = Uint128::new(data_struct.amount);
    
                    transfer_tokens(deps, account, token_address, amount)?;
                   
            }
        } 

         Ok(Response::default())      
        }






         //helper function to transfer tokens from contract to account
        pub fn transfer_tokens(
            deps: DepsMut,
            account: String,
            token_address: String,
            amount: Uint128,
        )  -> Result<Response,ContractError> {

            let account = Addr::unchecked(account);
            let token_address = Addr::unchecked(token_address);
            
           
           let current_balance = DEPOSITS.load(deps.storage,(&account,&token_address))?;

           if amount > current_balance {
            return Err(ContractError::InsufficentTokenBalance { token: token_address.to_string(), current_balance: amount })
           }

           let transfer_msg = &Cw20ExecuteMsg::Transfer { 
            recipient: account.to_string(),
             amount
             };

            let execute_msg = WasmMsg::Execute { 
                contract_addr: token_address.to_string(),
                 msg: to_binary(transfer_msg)?,
                  funds: vec![]
                 };

            let sub_msg = SubMsg::reply_always(execute_msg, SUCCESS_REPLY_MSG);  

            // Update state
            let current_balance = DEPOSITS.load(deps.storage, (&account, &token_address))?;
            DEPOSITS.save(deps.storage, (&account, &token_address), &(current_balance - amount))?;    

                 Ok(Response::new().add_submessage(sub_msg))
        }

}





#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}



#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        SUCCESS_REPLY_MSG => exec::reply_handler(deps, msg.result),
        _ => Err(StdError::generic_err("unknown reply id"))?,
    }
}





