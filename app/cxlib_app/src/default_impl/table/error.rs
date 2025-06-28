use cxlib_error_utils::MaybeFatalError;
use cxlib_internal::types::LoginError;
use redb::{CommitError, StorageError, TableError, TransactionError};

pub type StoreResult<T> = Result<T, StoreError>;
#[derive(thiserror::Error, Debug)]
pub enum StoreError {
    #[error("数据解析失败：`{0}`.")]
    ParseError(String),
    #[error(transparent)]
    LoginError(#[from] LoginError),
    #[error(transparent)]
    TableError(#[from] TableError),
    #[error(transparent)]
    StorageError(#[from] StorageError),
    #[error(transparent)]
    TransactionError(#[from] Box<TransactionError>),
    #[error(transparent)]
    CommitError(#[from] CommitError),
    #[error("在 `ReadTransaction` 上调用写操作。")]
    WriteOnReadTransaction,
    #[error("操作逻辑有误：`{0}`, 无法完成。")]
    LogicError(String),
    #[error("意外的空值：`{0}`, 无法完成。")]
    UnexpectedNone(String),
}
impl From<TransactionError> for StoreError {
    fn from(value: TransactionError) -> Self {
        Self::TransactionError(Box::new(value))
    }
}
impl MaybeFatalError for StoreError {
    fn is_fatal(&self) -> bool {
        let e = match self {
            StoreError::StorageError(e) => e,
            StoreError::TableError(TableError::Storage(e)) => e,
            StoreError::TableError(_) => {
                // match e {
                //     TableError::Storage(e) => e,
                //     // TableError::TableTypeMismatch { .. }
                //     // | TableError::TableIsMultimap(_)
                //     // | TableError::TableIsNotMultimap(_)
                //     // | TableError::TypeDefinitionChanged { .. } |
                //     // // 重命名时或打开只读表时出现。
                //     // TableError::TableDoesNotExist(_) |
                //     // // 只有在重命名时才会出现。
                //     // TableError::TableExists(_) |
                //     // TableError::TableAlreadyOpen(_, _) => { return true; }
                //     _ => return true,
                // }
                return true;
            }
            StoreError::ParseError(_) => return false,
            StoreError::LoginError(e) => {
                return e.is_fatal();
            }
            StoreError::TransactionError(e) => match &**e {
                TransactionError::Storage(e) => e,
                TransactionError::ReadTransactionStillInUse(_) => return false,
                _ => return true,
            },
            StoreError::CommitError(CommitError::Storage(e)) => e,
            StoreError::WriteOnReadTransaction => return false,
            _ => return true,
        };
        match e {
            StorageError::Corrupted(_)
            | StorageError::LockPoisoned(_)
            | StorageError::Io(_)
            | StorageError::PreviousIo
            | StorageError::ValueTooLarge(_) => true,
            _ => true,
        }
    }
}
