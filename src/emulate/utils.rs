pub fn to_u32_print(bytes: &[u8; 4]) -> u32 {
    let mut res: u32 = 0;
    for (i, b) in bytes.iter().enumerate() {
        res |= (*b as u32) << (8 * (3 - i));
    }
    res
}

pub fn to_u32_reg(bytes: &[u8; 4]) -> u32 {
    let mut res: u32 = 0;
    for (i, b) in bytes.iter().enumerate() {
        res |= (*b as u32) << (8 * i);
    }
    res
}

pub fn to_u8_slice(word: u32, bytes: &[u8]) {
    let mask = mask(8);
    for i in 0..4 {
        bytes[i] = ((word & mask) >> (8 * i)) as u8;
    }
}

pub fn extract_bit(word: &u32, index: u8) -> bool {
    word >> index & 1 == 1
}

pub fn extract_bits(word: &u32, pos: u8, size: u8) -> u32 {
    (word >> pos) & mask(size)
}

pub fn mask(size: u8) -> u32 {
    (1 << size) - 1
}

pub fn signed_24_to_32(num: i32) -> i32 {
    if extract_bit(&(num as u32), 23) {
        num | !mask(24) as i32
    } else {
        num
    }
}
