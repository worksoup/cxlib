use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("登录失败！")]
    LoginError(String),
    #[error(transparent)]
    AgentError(#[from] Box<ureq::Error>),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("`enc` 为空！")]
    EncError(String),
    #[error("无法获取位置信息！")]
    LocationError,
    #[error("二次验证信息为空！")]
    CaptchaEmptyError,
    #[error("设置位置预处理错误！")]
    SetLocationPreprocessorError,
}
