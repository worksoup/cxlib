use thiserror::Error;

pub type CxlibResult<T> = Result<T, crate::Error>;
#[derive(Error, Debug)]
pub enum Error {
    #[error("登录失败：`{0}`.")]
    LoginError(String),
    #[error("登录过期：`{0}`.")]
    LoginExpired(String),
    #[error(transparent)]
    AgentError(#[from] Box<ureq::Error>),
    #[error("功能已禁用：`{0}`.")]
    FunctionIsDisabled(String),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("`enc` 为空：`{0}`.")]
    EncError(String),
    #[error("无法获取位置信息！")]
    LocationError,
    #[error("验证出错：`{0}`.")]
    CaptchaError(String),
    #[error("设置位置预处理错误！")]
    SetLocationPreprocessorError,
    #[error("设置 CX 协议错误！")]
    SetProtocolError,
    #[error("数据解析失败：`{0}`.")]
    ParseError(String),
}
pub fn log_panic<T>(e: impl std::error::Error) -> T {
    log::error!("{}", e);
    panic!();
}

pub trait UnwrapOrLogPanic<T> {
    fn unwrap_or_log_panic(self) -> T;
}
impl<T, E: std::error::Error> UnwrapOrLogPanic<T> for Result<T, E> {
    fn unwrap_or_log_panic(self) -> T {
        self.unwrap_or_else(log_panic)
    }
}
