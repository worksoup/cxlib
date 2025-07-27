mod icon_click_image;
mod obstacle_image;
mod rotate_images;
mod slide_images;
mod text_click_info;

pub use icon_click_image::*;
pub use obstacle_image::*;
pub use rotate_images::*;
pub use slide_images::*;
pub use text_click_info::*;

pub trait SolverProviderTrait {
    type I: 'static;
    type O: 'static;
    fn solver(input: Self::I) -> Result<Self::O, crate::CaptchaError>;
}

use crate::CaptchaError;
use ureq::Agent;
pub trait VerificationInfoTrait: Sized {
    type I: 'static;
    type O: 'static;
    fn captcha_type() -> &'static str;
    /// 以自身的引用构造类型 `I`,
    /// 如：
    ///
    /// 验证信息可能包含图片 Url, 而计算验证结果需要图片类型，
    /// 则该函数需要做的应当为：下载图片并返回。
    fn prepare_data(self, agent: &Agent, referer: &str) -> Result<Self::I, CaptchaError>;
    /// 将结果转为字符串类型，用来向网站发送请求。
    fn result_to_string(result: Self::O) -> String;
    /// 过验证算法。如不实现则仅仅返回一个错误。
    fn default_solver(input: Self::I) -> Result<Self::O, CaptchaError> {
        let _ = input;
        Err(CaptchaError::UnsupportedType)
    }
    fn solve(self, agent: &Agent, referer: &str) -> Result<String, CaptchaError> {
        let data = self.prepare_data(agent, referer)?;
        let output = Self::default_solver(data)?;
        let r = Self::result_to_string(output);
        Ok(r)
    }
}
