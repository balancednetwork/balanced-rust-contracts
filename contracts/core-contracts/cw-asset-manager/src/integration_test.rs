
#![cfg(test)]

use cosmwasm_std::{coins, to_binary, Addr, Empty, Uint128};
use cw20::{Cw20Coin, Cw20Contract, Cw20ExecuteMsg};
use cw_multi_test::{App, Contract, ContractWrapper, Executor};

use cw_common::asset_manager_msg::*;




pub fn contract_assetmanager() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}


pub fn contract_cw20() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    );
    Box::new(contract)
}

#[test]

fn cw20_token_deposit() {

       
       let owner = Addr::unchecked("owner");

       let init_funds = coins(2000, "btc");
   
       let mut app = App::default();



   // set up cw20 contract with some tokens
   let cw20_id = app.store_code(contract_cw20());
   let msg = cw20_base::msg::InstantiateMsg {
       name: "Spokey".to_string(),
       symbol: "SPOK".to_string(),
       decimals: 18,
       initial_balances: vec![Cw20Coin {
           address: owner.to_string(),
           amount: Uint128::new(5000),
       }],
       mint: None,
       marketing: None,
   };
   let spok_addr = app
       .instantiate_contract(cw20_id, owner.clone(), &msg, &[], "SPOKE", None)
       .unwrap();



      // set up cw20 contract with some tokens
   let cw20_id = app.store_code(contract_cw20());
   let msg = cw20_base::msg::InstantiateMsg {
       name: "Hupper".to_string(),
       symbol: "HUP".to_string(),
       decimals: 18,
       initial_balances: vec![Cw20Coin {
           address: owner.to_string(),
           amount: Uint128::new(5000),
       }],
       mint: None,
       marketing: None,
   };
   let hupp_addr = app
       .instantiate_contract(cw20_id, owner.clone(), &msg, &[], "SPOKE", None)
       .unwrap();





//    setup asset manager contract 
   let am_id = app.store_code(contract_assetmanager());
   let msg = InstantiateMsg {
     x_call: Addr::unchecked("Deposit X-Call").into_string(),
     hub_address: Addr::unchecked("hub_address").into_string(),
     cw20_whitelist: vec![spok_addr.to_string(), hupp_addr.to_string()],

   };
   let adm_addr = app.instantiate_contract(
    am_id.clone(),
    owner.clone(),
    &msg,
    &[],
    "Asset Manager",
    None,
   );
   

   let config_msg = ExecuteMsg::ConfigureXcall {
     source_xcall: Addr::unchecked("hit Deposit").into_string(),
     destination_contract: Addr::unchecked("Destination-Contract").into_string(),
   };

   let exec_xcall_deposit = app.execute_contract(
    owner.clone(),
    adm_addr.unwrap(),
    &config_msg,
    &[], 
   ).unwrap();

   let deposit_msg =  Cw20ExecuteMsg::Transfer {
        recipient: spok_addr.to_string(),
        amount: Uint128::new(100),
   };

   let exec_deposit = app.execute_contract(
    owner.clone(),
    spok_addr,
    &deposit_msg,
    &[]
   ).unwrap();

//    match exec_deposit {
//     Ok(resp) => {
//         assert_eq!(resp.messages.len(), 2);
//     }
//     Err(e) => {
//         panic!("unexpected error occured: {:?}", e);
//     }
}


