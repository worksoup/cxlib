use crate::MaybeFatalError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("功能已禁用：`{0}`.")]
    FunctionIsDisabled(String),
    #[error("设置协议错误！")]
    SetProtocolError,
}

impl MaybeFatalError for ProtocolError {
    fn is_fatal(&self) -> bool {
        match self {
            ProtocolError::IoError(_) => false,
            ProtocolError::FunctionIsDisabled(_) => false,
            ProtocolError::SetProtocolError => false,
        }
    }
}
