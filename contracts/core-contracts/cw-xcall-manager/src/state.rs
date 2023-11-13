use cosmwasm_std::Addr;
use cw_common::xcall_manager_msg::ProtocolConfig;
use cw_storage_plus::Item;

pub const X_CALL: Item<String> = Item::new("xcall_address");
pub const ICON_GOVERNANCE: Item<String> = Item::new("icon_governance_network_address");
pub const PROPOSER: Item<Addr> = Item::new("admin_wallet");
pub const PROTOCOLS: Item<ProtocolConfig> = Item::new("protocols");
pub const PROPOSED_REMOVAL: Item<String> = Item::new("proposed_removal");
