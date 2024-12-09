use thiserror::Error;

#[derive(Error, Debug)]
#[error(transparent)]
pub struct AgentError(#[from] Box<ureq::Error>);
impl From<ureq::Error> for AgentError {
    fn from(value: ureq::Error) -> Self {
        Self(Box::new(value))
    }
}
#[derive(Error, Debug)]
pub enum LoginError {
    #[error(transparent)]
    AgentError(#[from] AgentError),
    #[error("登录失败，密码不符合规范：`{0}`.")]
    BadPassword(String),
    #[error(transparent)]
    CaptchaError(#[from] CaptchaError),
    #[error("Cookies 持久化失败：`{0}`.")]
    CookiesStoreError(Box<dyn std::error::Error + Send + Sync>),
    #[error("加解密错误：`{0}`.")]
    CryptoError(String),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("登录过期：`{0}`.")]
    LoginExpired(String),
    #[error("登录失败，服务器返回信息：`{0}`.")]
    ServerError(String),
    #[error("登录失败，不支持的登录协议。")]
    UnsupportedProtocol,
}
#[derive(Error, Debug)]
pub enum CaptchaError {
    #[error(transparent)]
    AgentError(#[from] AgentError),
    #[error("验证失败。")]
    VerifyFailed,
    #[error("不支持该类型的验证码。")]
    UnsupportedType,
    #[error("操作被主动取消：`{0}`.")]
    Canceled(String),
}
#[derive(Error, Debug)]
pub enum SignError {
    #[error(transparent)]
    AgentError(#[from] AgentError),
    #[error(transparent)]
    CaptchaError(#[from] CaptchaError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("无法获取位置信息：`{0}`")]
    LocationError(String),
    #[error("签到失败，所需信息未找到：`{0}`")]
    SignDataNotFound(String),
}
#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("功能已禁用：`{0}`.")]
    FunctionIsDisabled(String),
    #[error("设置协议错误！")]
    SetProtocolError,
}
#[derive(Error, Debug)]
pub enum StoreError {
    #[error("数据解析失败：`{0}`.")]
    ParseError(String),
    #[error(transparent)]
    LoginError(#[from] LoginError),
}
#[derive(Error, Debug)]
pub enum CourseError {
    #[error(transparent)]
    AgentError(#[from] AgentError),
    #[error(transparent)]
    LoginError(#[from] LoginError),
}
#[derive(Error, Debug)]
pub enum ActivityError {
    // #[error(transparent)]
    // AgentError(#[from] AgentError),
    // #[error(transparent)]
    // LoginError(#[from] LoginError),
}
// impl From<ureq::Error> for ActivityError {
//     fn from(value: ureq::Error) -> Self {
//         Self::AgentError(Box::new(value))
//     }
// }
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
