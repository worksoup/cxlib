use chrono::TimeDelta;
use log::warn;
use std::io::Read;
use std::ops::Add;
use std::time::{Duration, SystemTime};
use unicode_width::UnicodeWidthStr;

pub fn print_now() {
    let str = now_string();
    println!("{str}");
}
pub fn now_string() -> String {
    time_string(SystemTime::now())
}
pub fn time_string(t: SystemTime) -> String {
    chrono::DateTime::<chrono::Local>::from(t)
        .format("%+")
        .to_string()
}
pub fn time_string_from_mills(mills: u64) -> String {
    time_string(std::time::UNIX_EPOCH.add(Duration::from_millis(mills)))
}
pub fn time_delta_since_to_now(mills: u64) -> TimeDelta {
    let start_time = std::time::UNIX_EPOCH + Duration::from_millis(mills);
    let now = SystemTime::now();
    let duration = now.duration_since(start_time).unwrap();
    TimeDelta::from_std(duration).unwrap()
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
                warn!("输入的密码无法解析：{e}.");
                return None;
            }
        }
    })
}
pub fn get_width_str_should_be(s: &str, width: usize) -> usize {
    if UnicodeWidthStr::width(s) > width {
        width
    } else {
        UnicodeWidthStr::width(s) + 12 - s.len()
    }
}

pub fn zlib_encode(text: &str) -> Vec<u8> {
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use std::io::prelude::*;
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(text.as_bytes()).unwrap();
    encoder.finish().unwrap()
}
pub fn zlib_decode<R: Read>(r: R) -> String {
    let mut decoder = ZlibDecoder::new(r);
    use flate2::read::ZlibDecoder;
    let mut decompressed_data = String::new();
    decoder.read_to_string(&mut decompressed_data).unwrap();
    decompressed_data
}
pub fn find_qrcode_sign_enc_in_url(url: &str) -> Option<String> {
    // 在二维码图片中会有一个参数 `c`, 二维码预签到时需要。
    // 但是该参数似乎暂时可以从 `signDetail` 接口获取到。所以此处先注释掉。
    // let beg = r.find("&c=").unwrap();
    // let c = &r[beg + 3..beg + 9];
    // (c.to_owned(), enc.to_owned())
    // 有时二维码里没有参数，原因不明。
    let r = url
        .find("&enc=")
        .map(|beg| url[beg + 5..beg + 37].to_owned());
    if r.is_none() {
        warn!("{url:?}中没有找到二维码！")
    }
    r
}

#[cfg(test)]
mod test {
    #[test]
    fn test_des() {
        println!("{}", crate::now_string());
    }
}
