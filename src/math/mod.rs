pub fn two_bytes_to_word(h: u8, l: u8) -> u16 {
    let result: u16 = h as u16;
    return (result << 8) + (l as u16);
}

pub fn word_to_two_bytes(value: u16) -> (u8, u8) {
    let low: u8 = value as u8;
    let high: u8 = (value >> 8) as u8;

    return (high, low);
}
