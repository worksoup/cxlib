use crate::StoreError;
use cxlib_error_utils::MaybeFatalError;
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
impl MaybeFatalError for Error {
    #[inline]
    fn is_fatal(&self) -> bool {
        match self {
            Error::AgentError(agent_error) => agent_error.is_fatal(),
            Error::ActivityError(activity_error) => activity_error.is_fatal(),
            Error::CaptchaError(_) => false,
            Error::CourseError(course_error) => course_error.is_fatal(),
            Error::LoginError(login_error) => login_error.is_fatal(),
            Error::ProtocolError(_) => false,
            Error::SignError(sign_error) => sign_error.is_fatal(),
            Error::StoreError(store_error) => store_error.is_fatal(),
        }
    }
}
