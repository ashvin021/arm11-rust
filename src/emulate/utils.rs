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

pub fn to_u8_slice(word: u32) -> [u8; 4] {
    let mut bytes = [0; 4];
    for i in 0..4 {
        bytes[i] = (word >> (8 * i)) as u8;
    }
    bytes
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_u8_slice() {
        assert_eq!(&[0x2a, 0x01, 0x0, 0x0][..], &to_u8_slice(0x12a)[..]);
        assert_eq!(&[0xfa, 0x31, 0x21, 0x12][..], &to_u8_slice(0x122131fa)[..]);
    }
}
