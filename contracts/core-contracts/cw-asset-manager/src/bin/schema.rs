use cosmwasm_schema::write_api;

use cw_asset_manager::msg::{InstantiateMsg, QueryMsg};
use cw_common::asset_manager_msg::ExecuteMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
    }
}
