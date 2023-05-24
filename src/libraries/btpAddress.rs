use bytes::Bytes;
use cosmwasm_std::Addr;

mod BTPAddress {
    const PREFIX: Bytes = Bytes::from_static(b"btp://");
    const REVERT: String = "invalidBTPAddress";
    const DELIMITER: Bytes = Bytes::from_static(b"/");


    fn parseBTPAddress(_str:String) -> (String, String) {
        let _offset = validate(_str);
        return (_slice(_str, 0, _offset), _slice(_str, _offset+1, _str.len()));
    }

    fn networkAddress(_str:String) -> String {
        return _slice(_str,6,_validate(_str));
    }

    fn validate(_str:String) -> usize {
        let _offset = _str.find("/").unwrap_or(0);
        if _offset == 0 {
            panic!(REVERT);
        }
        return _offset;
    }
}