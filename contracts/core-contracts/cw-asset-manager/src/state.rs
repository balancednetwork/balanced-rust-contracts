use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

pub const OWNER: Item<Addr> = Item::new("contract_owner");
pub const VALID_TOKENS: Map<&Addr, bool> = Map::new("valid_tokens");

//equivalent to map(key:(depositor,token) -> amount)
pub const DEPOSITS: Map<(&Addr, &Addr), Uint128> = Map::new("deposits_of");

//map(user -> vec[token_addr])
pub const USER_TOKENS: Map<&Addr, Vec<Addr>> = Map::new("user_tokens");

//xcall deployed address on archway
pub const SOURCE_XCALL: Item<String> = Item::new("source_xcall_address");

pub const ICON_LOANS_ADDRESS: Item<String> = Item::new("destination_btp_address");
