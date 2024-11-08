use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("登录失败：{0}.")]
    LoginError(String),
    #[error("登录过期：{0}.")]
    LoginExpired(String),
    #[error(transparent)]
    AgentError(#[from] Box<ureq::Error>),
    #[error("功能已禁用：`{0}`.")]
    FunctionIsDisabled(String),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("`enc` 为空：{0}.")]
    EncError(String),
    #[error("无法获取位置信息！")]
    LocationError,
    #[error("二次验证信息为空！")]
    CaptchaEmptyError,
    #[error("设置位置预处理错误！")]
    SetLocationPreprocessorError,
    #[error("设置 CX 协议错误！")]
    SetProtocolError,
    #[error("数据解析失败：{0}.")]
    ParseError(String),
}
