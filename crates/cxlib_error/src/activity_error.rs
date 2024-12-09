use crate::{AgentError, CourseError, MaybeFatalError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ActivityError {
    #[error(transparent)]
    AgentError(#[from] AgentError),
    #[error(transparent)]
    CourseError(#[from] CourseError),
    // #[error(transparent)]
    // LoginError(#[from] LoginError),
}
impl MaybeFatalError for ActivityError {
    fn is_fatal(&self) -> bool {
        match self {
            ActivityError::AgentError(e) => e.is_fatal(),
            ActivityError::CourseError(e) => e.is_fatal(),
        }
    }
}
