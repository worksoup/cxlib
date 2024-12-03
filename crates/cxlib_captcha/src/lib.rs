use crate::utils::CaptchaType;
use onceinit::OnceInit;

mod hash;
pub mod protocol;
mod solver;
pub mod utils;
pub use solver::*;

pub type CaptchaId = String;

pub static DEFAULT_CAPTCHA_TYPE: OnceInit<CaptchaType> = OnceInit::new();
