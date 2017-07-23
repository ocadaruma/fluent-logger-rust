
pub fn i64_big_endian(i: i64, out: &mut [u8]) {
    out[0] = (i >> 56) as u8;
    out[1] = (i >> 48) as u8;
    out[2] = (i >> 40) as u8;
    out[3] = (i >> 32) as u8;
    out[4] = (i >> 24) as u8;
    out[5] = (i >> 16) as u8;
    out[6] = (i >> 8) as u8;
    out[7] = i as u8;
}

pub fn i64_little_endian(i: i64, out: &mut [u8]) {
    out[7] = (i >> 56) as u8;
    out[6] = (i >> 48) as u8;
    out[5] = (i >> 40) as u8;
    out[4] = (i >> 32) as u8;
    out[3] = (i >> 24) as u8;
    out[2] = (i >> 16) as u8;
    out[1] = (i >> 8) as u8;
    out[0] = i as u8;
}
