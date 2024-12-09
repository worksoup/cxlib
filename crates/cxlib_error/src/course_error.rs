use crate::{AgentError, LoginError, MaybeFatalError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CourseError {
    #[error(transparent)]
    AgentError(#[from] AgentError),
    #[error(transparent)]
    LoginError(#[from] LoginError),
}
/// 是否为致命错误。
///
/// 在程序中经常存在一些循环，循环中的语句可能产生错误。
/// 我们不知道究竟该直接返回还是打个日志。
///
/// 所以可以靠该特型进行区分，如果是致命错误则返回，不是则忽略。
///
/// 事实上，这里的“致命错误”实质上是循环中将会稳定复现的错误。
/// 比如网络环境错误等，整个循环周期内都稳定产生错误，此时应当视为
/// 致命错误，避免浪费时间。
impl MaybeFatalError for CourseError {
    fn is_fatal(&self) -> bool {
        match self {
            CourseError::AgentError(e) => e.is_fatal(),
            CourseError::LoginError(e) => e.is_fatal(),
        }
    }
}
