mod bin_code;

pub use bin_code::*;
use try_from_with_context::{FromStrWithContext, TryFromWithContext};

use log::warn;
use std::{borrow::Borrow, error::Error as ErrorTrait};

pub fn parse_lines<'a, 'cxt, T: FromStrWithContext<'a>, Cxt: Borrow<T::Context<'cxt>>>(
    contents: &'a str,
    metadata: Cxt,
) -> Vec<T>
where
    <T as TryFromWithContext<&'a str>>::Err: ErrorTrait,
{
    let contents = contents.split('\n');
    let mut line_count = 1_i64;
    let mut r = vec![];
    for line in contents {
        if !line.is_empty() {
            let data = FromStrWithContext::from_str(line, metadata.borrow());
            match data {
                Ok(data) => r.push(data),
                Err(e) => warn!("错误：第 {line_count} 行解析出错, 该行将被跳过！错误信息：{e}."),
            }
        }
        line_count += 1;
    }
    r
}

pub fn to_string_lines<T: ToString, Data: IntoIterator<Item = T>>(data: Data) -> String {
    let mut contents = String::new();
    let mut len = 0;
    for content in data.into_iter().map(|data| data.to_string()).enumerate() {
        len = content.0;
        contents += content.1.as_str();
        contents.push('\n');
    }
    if len == 0 {
        warn!("导出的数据为空，不做任何事情。")
    }
    contents
}
