use onceinit::OnceInit;

mod captcha_type;
mod hash;
mod solver;
pub mod utils;
mod verification_info;

pub use captcha_type::*;
pub use cxlib_error::CaptchaError;
pub use solver::*;
pub use verification_info::*;

pub type CaptchaId = String;

pub static DEFAULT_CAPTCHA_TYPE: OnceInit<CaptchaType> = OnceInit::new();
