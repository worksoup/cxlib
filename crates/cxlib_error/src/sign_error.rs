use crate::{AgentError, CaptchaError, MaybeFatalError};
use thiserror::Error;

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
impl MaybeFatalError for SignError {
    fn is_fatal(&self) -> bool {
        match self {
            SignError::AgentError(e) => e.is_fatal(),
            SignError::CaptchaError(e) => match e {
                CaptchaError::AgentError(e) => e.is_fatal(),
                CaptchaError::VerifyFailed => false,
                CaptchaError::UnsupportedType => true,
                CaptchaError::Canceled(_) => false,
            },
            SignError::IoError(_) => false,
            SignError::LocationError(_) => true,
            SignError::SignDataNotFound(_) => true,
        }
    }
}
