/// 交互式确认提示（是/否选择）
///
/// 此函数创建一个命令行确认对话框，支持本地化（中文"是/否"）和默认值指示。
/// 用户可输入多种格式的确认指令，包括中文和英文变体。
///
/// # 参数
/// - `inquire`: 主提示信息（如"是否继续？"）
/// - `tips`: 辅助帮助信息（显示在主提示下方）
///
/// # 返回值
/// 用户选择结果：
/// - `true` 表示确认（是）
/// - `false` 表示取消（否）
///
/// # 默认行为
/// - 默认选择"是"（按回车直接确认）
/// - 在选项中明确标记默认值
///
/// # 输入格式
/// 接受以下输入（不区分大小写）：
/// - 中文："是"、"否"
/// - 英文："y"、"yes"、"n"、"no"
/// - 空输入（使用默认值）
///
/// # 示例
/// ``` no_run
/// let should_continue = inquire_confirm("是否删除文件？", "此操作不可恢复");
/// if should_continue {
///     println!("执行删除操作");
/// }
/// ```
///
/// # 界面示例
/// ```text
/// 是否删除文件？ [是]/否
/// 此操作不可恢复
/// ```
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
/// 交互式密码输入
///
/// 安全获取密码输入，优先使用已有密码，若无则提示用户输入。
/// 输入时密码隐藏显示，不进行二次确认（适用于API密钥等场景）。
///
/// # 参数
/// - `pwd`: 可选密码：
///   - `Some(pwd)`：直接返回该密码（跳过用户输入）
///   - `None`：显示密码提示
///
/// # 返回值
/// - `Some(String)`：成功获取的密码
/// - `None`：用户输入错误或取消操作
///
/// # 错误处理
/// - 输入错误时记录警告日志（使用`log::warn!`）
/// - 返回`None`表示密码获取失败
///
/// # 示例
/// ```
/// // 场景1：使用预存密码
/// use cx_interact::inquire_pwd;
/// let api_key = inquire_pwd(Some("secret123".into()));
///
/// // 场景2：交互式输入
/// let password = inquire_pwd(None).expect("需要密码");
/// ```
///
/// # 界面示例
/// ```text
/// 密码： ********
/// ```
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
