#![cfg(test)]

use cosmwasm_std::{coins, to_binary, Addr, Empty, Uint128};
use cw20::{Cw20Coin, Cw20Contract, Cw20ExecuteMsg};
use cw_multi_test::{App, Contract, ContractWrapper, Executor};




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

       // set personal balance
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
   let SPOK_addr = app
       .instantiate_contract(cw20_id, owner.clone(), &msg, &[], "SPOKE", None)
       .unwrap();



   //setup asset manager contract 
   


}