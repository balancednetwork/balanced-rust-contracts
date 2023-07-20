use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::network_address::NetId;
use crate::network_address::NetworkAddress;
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    //executor address can be extracted at processing side
    Deposit {
        //TODO: normal archway validation
        token_address: String,
        amount: Uint128,
        to: Option<String>,
        data: Option<Vec<u8>>,
    },

    //TODO: introduce deposit transfer,
    // to field: network address(validation) to receive the (can be loans, another user address) (optional) defaults to caller
    // data field: depending upon the to address (optional)
    ConfigureXcall {
        source_xcall: String,
        //TODO: rename to destination asset manager
        destination_asset_manager: String,
    },

    HandleCallMessage {
        // _from the BTP address of the caller on the source chain
        from: String,

        // _data: The calldata delievered from the caller
        data: Vec<u8>,
    },
}

//#[derive(QueryResponses)]

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(OwnerResponse)]
    GetOwner {},
    #[returns(ConfiguredAddressesResponse)]
    GetConfiguredAddresses {},
    #[returns(NetworkAddressResponse)]
    GetNetworkAddress {},
    #[returns(NidResponse)]
    GetNid {},
    #[returns(IconAssetManagerResponse)]
    GetIconAssetManager {},
}

#[cw_serde]
pub struct OwnerResponse {
    pub owner: Addr,
}

#[cw_serde]
pub struct ConfiguredAddressesResponse {
    pub source_xcall: Addr,
    pub icon_asset_manager: NetworkAddress,
}

#[cw_serde]
pub struct NetworkAddressResponse {
    pub network_address: NetworkAddress,
}

#[cw_serde]
pub struct NidResponse {
    pub nid: NetId,
}

#[cw_serde]
pub struct IconAssetManagerResponse {
    pub icon_asset_manager: NetworkAddress,
}
