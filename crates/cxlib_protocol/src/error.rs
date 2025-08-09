use cxlib_error::AgentError;
use cxlib_error_utils::MaybeFatalError;

#[derive(thiserror::Error, Debug)]
pub enum ProtocolError {
    #[error(transparent)]
    AgentError(#[from] AgentError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("功能已禁用：`{0}`.")]
    FunctionIsDisabled(String),
    #[error("设置协议错误！")]
    SetProtocolError,
    #[error("数据解析失败：`{0}`.")]
    DataParseError(String),
}

impl From<ureq::Error> for ProtocolError {
    #[inline]
    fn from(value: ureq::Error) -> Self {
        Self::AgentError(AgentError::from(value))
    }
}
impl From<Box<ureq::Error>> for ProtocolError {
    #[inline]
    fn from(value: Box<ureq::Error>) -> Self {
        Self::AgentError(AgentError::from(value))
    }
}

impl MaybeFatalError for ProtocolError {
    #[inline]
    fn is_fatal(&self) -> bool {
        match self {
            ProtocolError::AgentError(agent_error) => agent_error.is_fatal(),
            ProtocolError::IoError(_) => false,
            ProtocolError::FunctionIsDisabled(_) => false,
            ProtocolError::SetProtocolError => false,
            ProtocolError::DataParseError(_) => false,
        }
    }
}
