use crate::{LoginError, MaybeFatalError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StoreError {
    #[error("数据解析失败：`{0}`.")]
    ParseError(String),
    #[error(transparent)]
    LoginError(#[from] LoginError),
}

impl MaybeFatalError for StoreError {
    fn is_fatal(&self) -> bool {
        match self {
            StoreError::ParseError(_) => false,
            StoreError::LoginError(e) => e.is_fatal(),
        }
    }
}
