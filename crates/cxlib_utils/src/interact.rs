
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
                log::warn!("输入的密码无法解析：{e}.");
                return None;
            }
        }
    })
}

pub fn get_width_str_should_be(s: &str, width: usize) -> usize {
    use unicode_width::UnicodeWidthStr;
    if UnicodeWidthStr::width(s) > width {
        width
    } else {
        UnicodeWidthStr::width(s) + 12 - s.len()
    }
}
