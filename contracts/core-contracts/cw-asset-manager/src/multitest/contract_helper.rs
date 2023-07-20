use cosmwasm_std::{Response,Addr,Uint128};
use cw_common::asset_manager_msg::ExecuteMsg;
use cw_multi_test::{App,Executor, AppResponse};
use crate::error::ContractError;

pub struct AssetManagerContract(pub Addr);

impl AssetManagerContract {
    pub fn addr(&self) -> &Addr {
        &self.0
    }

    #[track_caller]
    pub fn deposit(
        &self,
        sender: &Addr,
        app: &mut App,
        token: &String,
        amount: &Uint128,
        to: Option<&String>,
        data: Option<&Vec<u8>>,
    ) -> Result<AppResponse, ContractError> {
        let msg = ExecuteMsg::Deposit {
            token_address: token.clone(),
            amount: amount.clone(),
            to: to.map(|addr| addr.clone()),
            data: data.map(|data| data.clone()),
        };
        println!("msg: {:?} and am: {}",msg,self.0);
        app.execute_contract(sender.clone(), self.0.clone(), &msg, &[])
            .map_err(|err| err.downcast().unwrap())
    }
    
}

impl From<AssetManagerContract> for String {
    fn from(contract: AssetManagerContract) -> Self {
        contract.0.to_string()
    }
}







