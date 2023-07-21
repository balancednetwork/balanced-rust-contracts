use crate::{error::ContractError, state::SOURCE_XCALL};
use cosmwasm_std::{Addr, Uint128};
use cw_common::asset_manager_msg::ExecuteMsg;
use cw_multi_test::{App, AppResponse, Executor};

pub struct AssetManagerContract(pub Addr);
pub struct XCallContract(pub Addr);

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
        app.execute_contract(sender.clone(), self.0.clone(), &msg, &[])
            .map_err(|err| err.downcast().unwrap())
    }

    #[track_caller]
    pub fn configure_xcall(
        &self,
        sender: &Addr,
        app: &mut App,
        source_xcall: String, 
        destination_asset_manager: String,
    ) -> Result<AppResponse, ContractError> {
        let msg = ExecuteMsg::ConfigureXcall { source_xcall: source_xcall.clone(), destination_asset_manager: destination_asset_manager.clone()};

        app.execute_contract(sender.clone(), self.0.clone(), &msg, &[])
            .map_err(|err| err.downcast().unwrap())

        
    }
}

impl From<AssetManagerContract> for String {
    fn from(contract: AssetManagerContract) -> String {
        contract.0.to_string()
    }
}

// impl XCallContract {
//     pub fn addr(&self) -> &Addr {
//         &self.0
//     }

//     #[track_caller]
//     pub fn 

// }
