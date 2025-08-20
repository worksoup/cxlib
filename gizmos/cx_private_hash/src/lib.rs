// 混淆代码，不过仅仅在仓库中不可见。
// 如果你使用 IDE 或者对 rust 比较
// 熟悉的话应该很容易看到源码。
cx_obfuscate::__define!();
// let mut s = String::new();
// for i in (0..0x20 * a.len()).step_by(0x8) {
//     s.push(((a[i >> 5] >> (i % 0x20 & 0xff)) & 0xff) as u8 as char)
// }
// s
#[inline(always)]
fn to_bytes(a: [u32; 4]) -> [u8; 16] {
    #[cfg(target_endian = "big")]
    let a = [a[0].to_le(), a[1].to_le(), a[2].to_le(), a[3].to_le()];
    unsafe { std::mem::transmute(a) }
}
#[inline(always)]
fn hash__(a: impl AsRef<[u8]>) -> Vec<u32> {
    let a = a.as_ref();
    let mut array = vec![0_u32; (a.len() >> 2) + 1];
    for (index, byte) in a.iter().copied().enumerate() {
        let array_index = index >> 0x02;
        array[array_index] |= (0xff & byte as u32) << ((index * 8) % 0x20);
    }
    array
}
// 绝对会 panic.
fn _unused(a: &str, b: &str) -> [u8; 16] {
    let mut s = hash__(a);
    if 0x10 < s.len() {
        s = hash_(s, 0x08 * a.len()).to_vec();
    }
    let mut l = vec![0; 0x0f];
    let mut m = vec![0; 0x0f];
    for i in 0..0x10 {
        l[i] = 0x36363636 ^ s[i];
        m[i] = 0x5c5c5c5c ^ s[i];
    }
    l.append(&mut hash__(b));
    let a = hash_(l, 0x200 + 0x08 * b.len());
    m.append(&mut a.to_vec());
    to_bytes(hash_(m, 0x280))
}

#[inline(always)]
pub fn hash(a: impl AsRef<[u8]>) -> [u8; 16] {
    let a = a.as_ref();
    to_bytes(hash_(hash__(a), a.len()))
}

// let mut s = String::new();
// for c in a {
// s.push("0123456789abcdef".as_bytes()[((c >> 4) & 0x0f_u8) as usize] as char);
// s.push("0123456789abcdef".as_bytes()[(c & 0x0f_u8) as usize] as char);
// }
// s

#[cfg(test)]
mod tests {
    use crate::{hash_, hash__, to_bytes};

    // #[test]
    // fn it_works() {
    //     print!("[");
    //     for k in C {
    //         let k = k as u32;
    //         print!("0x{k:02x}, ");
    //     }
    //     println!("]");
    // }
    #[test]
    fn f() {
        print!("[");
        let mut i = 0;
        for _ in 0..16 {
            print!("0x{i:02x}, ");
            i += 1;
        }
        i = 1;
        for _ in 0..16 {
            print!("0x{i:02x}, ");
            i = (i + 5) % 16;
        }
        i = 5;
        for _ in 0..16 {
            print!("0x{i:02x}, ");
            i = (i + 3) % 16;
        }
        i = 0;
        for _ in 0..16 {
            print!("0x{i:02x}, ");
            i = (i + 7) % 16;
        }
        println!("]");
    }
    #[test]
    fn pre_hash_test() {
        let k = "12121212";
        println!("{:?}", hash__(k));
        let k = hash_(hash__(k), k.len());
        // println!("{:?}", 0x0e + (((32 + 64) >> 9) << 4));
        // println!("{:?}", 128 << 32 % 32);
        assert_eq!(
            to_bytes(k),
            [
                140, 232, 123, 142, 195, 70, 255, 76, 128, 99, 95, 102, 125, 21, 146, 174
            ]
        );
    }
}
