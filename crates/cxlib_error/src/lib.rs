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

pub trait UnwrapOrLogPanic<T> {
    fn unwrap_or_log_panic(self) -> T;
}
impl<T, E: std::error::Error> UnwrapOrLogPanic<T> for Result<T, E> {
    fn unwrap_or_log_panic(self) -> T {
        self.unwrap_or_else(log_panic)
    }
}
