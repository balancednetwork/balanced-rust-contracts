use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
struct NetworkAddress {
    address: String,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum XCallQuery {
    //later in case of multiplre bridges
    //     #[returns(NetworkAddress)]
    //     GetNetworkAddress {
    //         protocol: String,
    //         network_id: String,
    //      },
    // }
    #[returns(NetworkAddress)]
    GetNetworkAddress {},
}

#[cw_serde]
pub enum XCallMsg {
    // @param data represent hex encoded string / hex encoded number in vec
    // @param to represent the callee contract which recieve the data on the destination chain
    // @param rollback is for handling error cases. 
    SendCallMessage {

        //  The BTP address of the callee on  the destination chain
        to: String,

        // The calldata specific to the target contract
        data: Vec<u8>,

        // (optional) The data for restoring the caller state when an error occured
        rollback: Option<Vec<u8>>,

        // used for returning the serial number of the request
    },
}
