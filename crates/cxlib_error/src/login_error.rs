use crate::{new_types::AgentError, CaptchaError, MaybeFatalError};
use thiserror::Error;

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
impl MaybeFatalError for LoginError {
    fn is_fatal(&self) -> bool {
        match self {
            LoginError::AgentError(e) => e.is_fatal(),
            LoginError::BadPassword(_) => false,
            LoginError::CaptchaError(e) => match e {
                CaptchaError::AgentError(e) => e.is_fatal(),
                CaptchaError::VerifyFailed => false,
                CaptchaError::UnsupportedType => true,
                CaptchaError::Canceled(_) => false,
                CaptchaError::RequestRefresh => false,
            },
            LoginError::CookiesStoreError(_) => false,
            LoginError::CryptoError(_) => false,
            LoginError::IoError(_) => false,
            LoginError::LoginExpired(_) => false,
            LoginError::ServerError(_) => false,
            LoginError::UnsupportedProtocol => false,
        }
    }
}
