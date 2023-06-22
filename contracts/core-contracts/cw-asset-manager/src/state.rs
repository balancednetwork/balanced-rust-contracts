use cosmwasm_std::{Addr,Uint128};
use cw_storage_plus::{Item,Map};




pub const OWNER: Item<Addr> = Item::new("contract_owner");
pub const VALID_TOKENS: Map<&Addr,bool> = Map::new("valid_tokens");

//equivalent to map(key:(depositor,token) -> amount)
pub const DEPOSITS: Map<(&Addr,&Addr), Uint128> = Map::new("deposits_of");

//map(user -> vec[token_addr])
pub const USER_TOKENS: Map<&Addr,Vec<Addr>> = Map::new("user_tokens");





pub const X_CALL: Item<String> = Item::new("xCall");
pub const X_CALL_BTP_ADDRESS: Item<String> = Item::new("xCallBTPAddress");
pub const NID: Item<String> = Item::new("nid");
pub const HUB_ADDRESS: Item<String> = Item::new("hubAddress");
pub const HUB_NET: Item<String> = Item::new("hubNet");



