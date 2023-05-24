mod String {
    
    fn bytes_to_hex(buffer: &[u8]) -> String {
        if buffer.is_empty() {
            return String::from("0x");
        }
    
        let hex_chars = "0123456789abcdef".as_bytes();
    
        let mut hex_string = String::with_capacity(buffer.len() * 2 + 2);
        hex_string.push_str("0x");
    
        for &byte in buffer {
            let high_nibble = (byte >> 4) & 0x0F;
            let low_nibble = byte & 0x0F;
            hex_string.push(hex_chars[high_nibble as usize] as char);
            hex_string.push(hex_chars[low_nibble as usize] as char);
        }
    
        hex_string
    }

    fn concat(_base: &str, _value: &str) -> String {
        let concatenated = _base+_value;
        concatenated
    }
    
    fn _index_of(_base: &str, _value: &str, _offset: usize) -> Option<i64> {
        let _base_bytes = _base.as_bytes();
        let _value_bytes = _value.as_bytes();
    
        assert_eq!(_value_bytes.len(), 1);
    
        for i in _offset.._base_bytes.len() {
            if _base_bytes[i] == _value_bytes[0] {
                return Some(i as i64);
            }
        }
    
        None
    }


    
}