use bytes::Bytes;

mod Types {

    struct CrossTransfer {
        from: String,
        to: String,
        value: u128,
        data: Bytes,
    }

    struct CrossTransferRevert {
        from: String,
        value: u128,
    }


}