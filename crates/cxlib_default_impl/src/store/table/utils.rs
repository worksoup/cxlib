use std::{error::Error as ErrorTrait, str::FromStr};

use log::warn;

pub fn parse<E: ErrorTrait, T: FromStr<Err = E>>(contents: &str) -> Vec<T> {
    let contents = contents.split('\n');
    let mut line_count = 1_i64;
    let mut r = vec![];
    for line in contents {
        if !line.is_empty() {
            let data = line.trim().parse();
            match data {
                Ok(data) => r.push(data),
                Err(e) => warn!("错误：第 {line_count} 行解析出错, 该行将被跳过！错误信息：{e}."),
            }
        }
        line_count += 1;
    }
    r
}

pub fn to_string<T: ToString, Data: Iterator<Item = T>>(data: Data) -> String {
    let mut contents = String::new();
    let mut len = 0;
    for content in data.map(|data| data.to_string()).enumerate() {
        len = content.0;
        contents += content.1.as_str();
        contents.push('\n');
    }
    if len == 0 {
        warn!("导出的数据为空，不做任何事情。")
    }
    contents
}
