#![allow(unused_imports)]
pub use interact::*;
mod interact {
    #[cfg(feature = "inquire")]
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
    #[cfg(feature = "inquire")]
    pub fn inquire_pwd(pwd: Option<String>) -> Option<String> {
        Some(if let Some(pwd) = pwd {
            pwd
        } else {
            match inquire::Password::new("密码：")
                .without_confirmation()
                .prompt()
            {
                Ok(pwd) => pwd,
                Err(e) => {
                    #[cfg(feature = "log")]
                    log::warn!("输入的密码无法解析：{e}.");
                    #[cfg(not(feature = "log"))]
                    eprintln!("输入的密码无法解析：{e}.");
                    return None;
                }
            }
        })
    }
    #[cfg(feature = "unicode-width")]
    pub fn get_width_str_should_be(s: &str, width: usize) -> usize {
        use unicode_width::UnicodeWidthStr;
        if UnicodeWidthStr::width(s) > width {
            width
        } else {
            UnicodeWidthStr::width(s) + 12 - s.len()
        }
    }
}
pub use date_time::*;
mod date_time {
    use std::time::{Duration, SystemTime};
    #[cfg(feature = "chrono")]
    pub fn print_now() {
        let str = now_string();
        println!("{str}");
    }
    #[cfg(feature = "chrono")]
    pub fn now_string() -> String {
        time_string(SystemTime::now())
    }
    #[cfg(feature = "chrono")]
    pub fn time_string(t: SystemTime) -> String {
        chrono::DateTime::<chrono::Local>::from(t)
            .format("%+")
            .to_string()
    }
    #[cfg(feature = "chrono")]
    pub fn time_string_from_mills(mills: u64) -> String {
        time_string(std::time::UNIX_EPOCH + Duration::from_millis(mills))
    }
    #[cfg(feature = "chrono")]
    pub fn time_delta_since_to_now(mills: u64) -> chrono::TimeDelta {
        let start_time = std::time::UNIX_EPOCH + Duration::from_millis(mills);
        let now = SystemTime::now();
        let duration = now.duration_since(start_time).unwrap();
        chrono::TimeDelta::from_std(duration).unwrap()
    }
}
pub use crypto::*;
mod crypto {
    #[cfg(feature = "pkcs7_pad")]
    pub fn pkcs7_pad<const BLOCK_SIZE: usize>(data: &[u8]) -> Vec<[u8; BLOCK_SIZE]> {
        let len = data.len();
        let batch = len / BLOCK_SIZE;
        let m = len % BLOCK_SIZE;
        let len2 = BLOCK_SIZE - m;
        let mut r = vec![[0u8; BLOCK_SIZE]; batch + 1];
        let pad_num = ((BLOCK_SIZE - m) % 0xFF) as u8;
        let r_data = r.as_mut_ptr() as *mut u8;
        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), r_data, len);
            std::ptr::copy_nonoverlapping(
                vec![pad_num; len2].as_ptr(),
                r_data.add(batch * BLOCK_SIZE + m),
                len2,
            );
        }
        r
    }
    pub use des_impl::*;
    mod des_impl {
        #[cfg(feature = "des_impl")]
        use des::{
            cipher::{generic_array::GenericArray, BlockEncrypt as _, KeyInit as _},
            Des,
        };
        #[cfg(feature = "des_impl")]
        use crate::pkcs7_pad;

        #[cfg(feature = "des_impl")]
        pub fn des_enc(data: &[u8], key: [u8; 8]) -> String {
            let key = GenericArray::from(key);
            let des = Des::new(&key);
            let mut data_block_enc = Vec::new();
            for block in pkcs7_pad(data) {
                let mut block = GenericArray::from(block);
                des.encrypt_block(&mut block);
                let mut block = block.to_vec();
                data_block_enc.append(&mut block);
            }
            hex::encode(data_block_enc)
        }
    }
    pub use zlib_impl::*;
    mod zlib_impl {
        use std::io::Read;
        #[cfg(feature = "zlib_impl")]
        pub fn zlib_encode(text: &str) -> Vec<u8> {
            use flate2::write::ZlibEncoder;
            use flate2::Compression;
            use std::io::prelude::*;
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(text.as_bytes()).unwrap();
            encoder.finish().unwrap()
        }
        #[cfg(feature = "zlib_impl")]
        pub fn zlib_decode<R: Read>(r: R) -> String {
            let mut decoder = ZlibDecoder::new(r);
            use flate2::read::ZlibDecoder;
            let mut decompressed_data = String::new();
            decoder.read_to_string(&mut decompressed_data).unwrap();
            decompressed_data
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_des() {
        #[cfg(feature = "interact")]
        println!("{}", crate::now_string());
    }
}
