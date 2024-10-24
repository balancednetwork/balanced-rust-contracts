#[cfg(feature = "injective")]
use super::cw20_adapter::CW20Adapter;
use cosmwasm_std::Addr;
use cw_common::network_address::{NetId, NetworkAddress};
use cw_storage_plus::Item;

pub const OWNER: Item<Addr> = Item::new("owner");
pub const X_CALL: Item<Addr> = Item::new("xCall");
pub const X_CALL_MANAGER: Item<Addr> = Item::new("xcall_manager");
pub const X_CALL_NETWORK_ADDRESS: Item<NetworkAddress> = Item::new("xCallBTPAddress");
pub const NID: Item<NetId> = Item::new("nid");
pub const DESTINATION_TOKEN_ADDRESS: Item<Addr> = Item::new("hubAddress");
pub const DESTINATION_TOKEN_NET: Item<NetId> = Item::new("hubNet");

#[cfg(feature = "injective")]
pub const CW20_ADAPTER: Item<CW20Adapter> = Item::new("cw20_adapter");
