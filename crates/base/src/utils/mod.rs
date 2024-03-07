use des::{
    cipher::{generic_array::GenericArray, BlockEncrypt, KeyInit},
    Des,
};
use directories::ProjectDirs;
use lazy_static::lazy_static;
use std::path::PathBuf;
use unicode_width::UnicodeWidthStr;
lazy_static! {
    pub static ref CONFIG_DIR: PathBuf = {
        let is_testing = std::env::var("TEST_CXSIGN").is_ok();
        let binding = ProjectDirs::from("rt.lea", "worksoup", "newsign").unwrap();
        let dir = if is_testing {
            binding.config_dir().join("test").to_owned()
        } else {
            binding.config_dir().to_owned()
        };
        let _ = std::fs::create_dir_all(dir.clone());
        dir
    };
}

pub fn print_now() {
    let str = chrono::DateTime::<chrono::Local>::from(std::time::SystemTime::now())
        .format("%+")
        .to_string();
    println!("{str}");
}

pub fn inquire_confirm(inquire: &str, tips: &str) -> bool {
    inquire::Confirm::new(inquire)
        .with_help_message(tips)
        .with_default_value_formatter(&|v| if v { "是[默认]" } else { "否[默认]" }.into())
        .with_formatter(&|v| if v { "是" } else { "否" }.into())
        .with_parser(&|s| match inquire::Confirm::DEFAULT_PARSER(s) {
            r @ Ok(_) => r,
            Err(_) => {
                if s == "是" {
                    Ok(true)
                } else if s == "否" {
                    Ok(false)
                } else {
                    Err(())
                }
            }
        })
        .with_error_message("请以\"y\", \"yes\"等表示“是”，\"n\", \"no\"等表示“否”。")
        .with_default(true)
        .prompt()
        .unwrap()
}

pub fn des_enc(text: &str) -> String {
    fn pkcs7(text: &str) -> Vec<[u8; 8]> {
        assert!(text.len() > 7);
        assert!(text.len() < 17);
        let mut r = Vec::new();
        let pwd = text.as_bytes();
        let len = pwd.len();
        let batch = len / 8;
        let m = len % 8;
        for i in 0..batch {
            let mut a = [0u8; 8];
            a.copy_from_slice(&pwd[i * 8..8 + i * 8]);
            r.push(a);
        }
        let mut b = [0u8; 8];
        for i in 0..m {
            b[i] = pwd[8 * batch + i];
        }
        for item in b.iter_mut().skip(m) {
            *item = (8 - m) as u8;
        }
        r.push(b);
        // #[cfg(debug_assertions)]
        // println!("{r:?}");
        r
    }
    let key = b"u2oh6Vu^".to_owned();
    let key = GenericArray::from(key);
    let des = Des::new(&key);
    let mut data_block_enc = Vec::new();
    for block in pkcs7(text) {
        let mut block = GenericArray::from(block);
        des.encrypt_block(&mut block);
        let mut block = block.to_vec();
        data_block_enc.append(&mut block);
    }
    hex::encode(data_block_enc)
}

pub fn get_width_str_should_be(s: &str, width: usize) -> usize {
    if UnicodeWidthStr::width(s) > width {
        width
    } else {
        UnicodeWidthStr::width(s) + 12 - s.len()
    }
}

// mod test {
//     #[test]
//     fn test_des() {
//         println!("{}", crate::utils::pwd_des("0123456789."));
//     }
// }
