pub fn to_u32_print(bytes: &[u8; 4]) -> u32 {
    let mut res: u32 = 0;
    for (i, b) in bytes.iter().enumerate() {
        res |= (*b as u32) << 8 * (3 - i);
    }
    res
}

pub fn to_u32_reg(bytes: &[u8; 4]) -> u32 {
    let mut res: u32 = 0;
    for (i, b) in bytes.iter().enumerate() {
        res |= (*b as u32) << 8 * i;
    }
    res
}
