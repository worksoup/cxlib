use rand::Rng;

pub(crate) use cx_private_hash::hash;

// let mut s = String::new();
// for c in a {
// s.push("0123456789abcdef".as_bytes()[((c >> 4) & 0x0f_u8) as usize] as char);
// s.push("0123456789abcdef".as_bytes()[(c & 0x0f_u8) as usize] as char);
// }
// s
#[inline(always)]
pub(crate) fn encode(a: [u8; 16]) -> String {
    hex::encode(a)
}
#[inline]
pub(crate) fn uuid() -> String {
    let mut v = [0; 0x24];
    #[inline(always)]
    fn fill_hex<T>(_: T) -> u8 {
        let index = rand::rng().random_range(0x00..0x10) as usize;
        b"0123456789abcdef"[index]
    }
    v = v.map(fill_hex);
    v[0x0e] = b'4';
    let i = (0x3 & if v[0x13].is_ascii_digit() { v[0x13] } else { 0 }) | 0x8;
    v[0x13] = b"0123456789abcdef"[i as usize];
    for i in [0x08, 0x0d, 0x12, 0x17] {
        v[i] = b'-';
    }
    unsafe { String::from_utf8_unchecked(v.into()) }
}

#[cfg(test)]
mod tests {
    use cx_private_hash::hash;

    use crate::hash::{encode, uuid};
    #[test]
    fn pre_hash_test() {
        let k = "12121212";
        let k = hash(k);
        assert_eq!(encode(k), "8ce87b8ec346ff4c80635f667d1592ae");
        let k = k.map(|a| a as i32);
        assert_eq!(
            k,
            [
                140, 232, 123, 142, 195, 70, 255, 76, 128, 99, 95, 102, 125, 21, 146, 174
            ]
        );
        let u = uuid();
        println!("{u:?}");
    }
}
