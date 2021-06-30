pub fn extract_bit(word: &u32, index: u8) -> bool {
    ((word >> index) & 1) == 1
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
