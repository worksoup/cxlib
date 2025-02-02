pub use cxlib_captcha as captcha;
pub use cxlib_default_impl as default_impl;
pub use cxlib_imageproc as imageproc;
pub use cxlib_login as login;
pub use cxlib_protocol as protocol;
pub use cxlib_sign as sign;
pub use cxlib_store as store;
pub mod types {
    pub use cxlib_types::*;
    pub use cxlib_types_ext as ext;
}
pub use cxlib_utils as utils;

pub mod error {
    use cxlib_error::*;
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
        InitError(#[from] InitError),
        #[error(transparent)]
        LoginError(#[from] LoginError),
        #[error(transparent)]
        ProtocolError(#[from] ProtocolError),
        #[error(transparent)]
        SignError(#[from] SignError),
        #[error(transparent)]
        StoreError(#[from] StoreError),
    }
}
