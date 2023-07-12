use cosmwasm_std::Addr;
use cw_common::network_address::{NetId, NetworkAddress};
use cw_storage_plus::Item;

pub const OWNER: Item<Addr> = Item::new("contract_owner");

pub const SOURCE_XCALL: Item<String> = Item::new("source_xcall_address");
pub const X_NETWORK_ADDRESS: Item<NetworkAddress> = Item::new("source_xcall_neaddress");
pub const NID: Item<NetId> = Item::new("network_id");

pub const ICON_ASSET_MANAGER: Item<NetworkAddress> =
    Item::new("icon_asset_manager_network_address");
pub const ICON_NET_ID: Item<NetId> = Item::new("icon_asset_manager_network_id");
