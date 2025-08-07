/// 将每个16位半字互换
pub fn swap_half_word_byte_sequence_u32(value: u32) -> u32 {
    ((value & 0x0000FFFF) << 16) | ((value & 0xFFFF0000) >> 16)
}

/// 将u32的4个字节逆序排列
pub fn swap_word_byte_sequence_u32(value: u32) -> u32 {
    ((value & 0x000000FF) << 24)
        | ((value & 0x0000FF00) << 8)
        | ((value & 0x00FF0000) >> 8)
        | ((value & 0xFF000000) >> 24)
}

// pub fn u8_to_u32_slice(bytes: &Vec<u8>) -> Vec<u32> {
//     assert!(bytes.len() % 4 == 0, "字节数组长度必须是4的倍数");

//     let mut result = Vec::with_capacity(bytes.len() / 4);

//     // 每次处理4个字节
//     for chunk in bytes.chunks_exact(4) {
//         let value = u32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
//         result.push(value);
//     }

//     result
// }
