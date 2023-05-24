mod RLPEncode {
    use cosmwasm_std::Uint128;

    const NULL: [u8; 2] = [0xf8, 0x00];
    
    const MAX_INT8: i8 = i8::MAX;
    const MAX_INT16: i16 = i16::MAX;
    const MAX_INT24: i32 = i32::MAX;
    const MAX_INT32: i32 = i32::MAX;
    const MAX_INT40: i64 = i64::MAX;
    const MAX_INT48: i64 = i64::MAX;
    const MAX_INT56: i64 = i64::MAX;
    const MAX_INT64: i64 = i64::MAX;
    const MAX_INT72: i128 = i128::MAX;
    const MAX_INT80: i128 = i128::MAX;
    const MAX_INT88: i128 = i128::MAX;
    const MAX_INT96: i128 = i128::MAX;
    const MAX_INT104: i128 = i128::MAX;
    const MAX_INT112: i128 = i128::MAX;
    const MAX_INT120: i128 = i128::MAX;
    const MAX_INT128: i128 = i128::MAX;
    
    const MAX_UINT8: u8 = u8::MAX;
    const MAX_UINT16: u16 = u16::MAX;
    const MAX_UINT24: u32 = u32::MAX;
    const MAX_UINT32: u32 = u32::MAX;
    const MAX_UINT40: u64 = u64::MAX;
    const MAX_UINT48: u64 = u64::MAX;
    const MAX_UINT56: u64 = u64::MAX;
    const MAX_UINT64: u64 = u64::MAX;
    const MAX_UINT72: Uint128 = Uint128::MAX;
    const MAX_UINT80: Uint128 = Uint128::MAX;
    const MAX_UINT88: Uint128 = Uint128::MAX;
    const MAX_UINT96: Uint128 = Uint128::MAX;
    const MAX_UINT104: Uint128 = Uint128::MAX;
    const MAX_UINT112: Uint128 = Uint128::MAX;
    const MAX_UINT120: Uint128 = Uint128::MAX;
    const MAX_UINT128: Uint128 = Uint128::MAX;

    // pub fn encodeBytes()   
}
