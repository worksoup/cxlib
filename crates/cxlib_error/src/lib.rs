mod activity_error;
mod captcha_error;
mod course_error;
mod login_error;
mod new_types;
mod protocol_error;
mod sign_error;
mod store_error;

pub use activity_error::*;
pub use captcha_error::*;
pub use course_error::*;
pub use login_error::*;
pub use new_types::*;
pub use protocol_error::*;
pub use sign_error::*;
pub use store_error::*;
pub trait MaybeFatalError {
    fn is_fatal(&self) -> bool;
}

pub fn log_panic<T>(e: impl std::error::Error) -> T {
    log::error!("{}", e);
    panic!();
}

pub fn log_default<T: Default>(e: impl std::error::Error) -> T {
    log::warn!("{}", e);
    T::default()
}

pub trait CxlibResultUtils<T> {
    fn log_unwrap(self) -> T;
    fn unwrap_or_log_default(self) -> T
    where
        T: Default;
}
impl<T, E: std::error::Error> CxlibResultUtils<T> for Result<T, E> {
    fn log_unwrap(self) -> T {
        self.unwrap_or_else(log_panic)
    }

    fn unwrap_or_log_default(self) -> T
    where
        T: Default,
    {
        self.unwrap_or_else(log_default)
    }
}
