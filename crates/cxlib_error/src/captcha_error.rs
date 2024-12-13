use crate::{AgentError, MaybeFatalError};
use thiserror::Error;

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
    #[error("需要刷新。")]
    RequestRefresh,
}
/// 注意，此处的 Canceled 是用户取消，仅在重试循环中视为致命错误。
impl MaybeFatalError for CaptchaError {
    fn is_fatal(&self) -> bool {
        match self {
            CaptchaError::AgentError(e) => e.is_fatal(),
            CaptchaError::VerifyFailed => false,
            CaptchaError::UnsupportedType => true,
            CaptchaError::Canceled(_) => true,
            CaptchaError::RequestRefresh => false,
        }
    }
}
