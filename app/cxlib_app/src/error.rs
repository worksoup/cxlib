use crate::StoreError;
use cxlib_internal::{
    captcha::CaptchaError,
    protocol::{AgentError, ProtocolError},
    sign::SignError,
    types::{ActivityError, CourseError, LoginError},
};

pub type CxlibResult<T> = Result<T, Error>;
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    AgentError(#[from] AgentError),
    #[error(transparent)]
    ActivityError(#[from] ActivityError),
    #[error(transparent)]
    CaptchaError(#[from] CaptchaError),
    #[error(transparent)]
    CourseError(#[from] CourseError),
    #[error(transparent)]
    LoginError(#[from] LoginError),
    #[error(transparent)]
    ProtocolError(#[from] ProtocolError),
    #[error(transparent)]
    SignError(#[from] SignError),
    #[error(transparent)]
    StoreError(#[from] StoreError),
}
