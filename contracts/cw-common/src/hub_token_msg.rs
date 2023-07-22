use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128, Binary};
use cw20::Expiration;

use crate::network_address::NetworkAddress;
pub use cw20_base::msg::{QueryMsg ,ExecuteMsg as Cw20ExecuteMsg};


#[cw_serde]
pub struct InstantiateMsg {
    pub x_call: String,
    pub hub_address: String,
}

//TODO: Add network address as a parameter for xcall network address
#[cw_serde]
pub enum ExecuteMsg {
    Setup {
        //TODO: x_call should be of addr type
        x_call: Addr,
        hub_address: NetworkAddress,
    },
    HandleCallMessage {
        from: NetworkAddress,
        data: Vec<u8>,
    },
    CrossTransfer {
        to: NetworkAddress,
        amount: u128,
        data: Vec<u8>,
    },
    /// Transfer is a base message to move tokens to another account without triggering actions
    Transfer { recipient: String, amount: Uint128 },
    /// Burn is a base message to destroy tokens forever
    Burn { amount: Uint128 },
    /// Send is a base message to transfer tokens to a contract and trigger an action
    /// on the receiving contract.
    Send {
        contract: String,
        amount: Uint128,
        msg: Binary,
    },
    /// Only with "approval" extension. Allows spender to access an additional amount tokens
    /// from the owner's (env.sender) account. If expires is Some(), overwrites current allowance
    /// expiration with this one.
    IncreaseAllowance {
        spender: String,
        amount: Uint128,
        expires: Option<Expiration>,
    },
    /// Only with "approval" extension. Lowers the spender's access of tokens
    /// from the owner's (env.sender) account by amount. If expires is Some(), overwrites current
    /// allowance expiration with this one.
    DecreaseAllowance {
        spender: String,
        amount: Uint128,
        expires: Option<Expiration>,
    },
    /// Only with "approval" extension. Transfers amount tokens from owner -> recipient
    /// if `env.sender` has sufficient pre-approval.
    TransferFrom {
        owner: String,
        recipient: String,
        amount: Uint128,
    },
    /// Only with "approval" extension. Sends amount tokens from owner -> contract
    /// if `env.sender` has sufficient pre-approval.
    SendFrom {
        owner: String,
        contract: String,
        amount: Uint128,
        msg: Binary,
    },
    /// Only with "approval" extension. Destroys tokens forever
    BurnFrom { owner: String, amount: Uint128 },
    /// Only with the "mintable" extension. If authorized, creates amount new tokens
    /// and adds to the recipient balance.
    Mint { recipient: String, amount: Uint128 },
    /// Only with the "mintable" extension. The current minter may set
    /// a new minter. Setting the minter to None will remove the
    /// token's minter forever.
    UpdateMinter { new_minter: Option<String> },
}

