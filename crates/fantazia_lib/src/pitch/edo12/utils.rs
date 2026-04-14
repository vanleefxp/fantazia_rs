use num_integer::Integer as _;

const BLACK_KEY_MASK: u16 = 0b0101010_01010;
pub const KEY_IDX_OF_COLOR: [u8; 12] = [0, 0, 1, 1, 2, 3, 2, 4, 3, 5, 4, 6];
pub const WHITE_TONES: [u8; 7] = [0, 2, 4, 5, 7, 9, 11];
pub const BLACK_TONES: [u8; 5] = [1, 3, 6, 8, 10];
/// White key index on the left side of each black key.
pub const NEIGHBORING_WHITE_KEYS: [u8; 5] = [0, 1, 3, 4, 5];

#[inline(always)]
pub fn is_black_key_otone(otone: u8) -> bool {
    BLACK_KEY_MASK & (1 << otone) != 0
}

#[inline(always)]
pub fn is_black_key(key: i8) -> bool {
    is_black_key_otone(key.mod_floor(&12) as u8)
}