use cxlib_error::AgentError;
use log::debug;
use std::fmt::Display;
use ureq::{Agent, Body, http::Response};
// Doesn't matter.
pub static CALLBACK_NAME: &str = "cx_captcha_function";
static VERSION_PARAM: &str = "version=1.1.20";
pub static CAPTCHA_ID: &str = "Qt9FIw9o4pwRjOyqM6yizZBh682qN2TU";
pub trait CaptchaProtocolTrait {
    fn get_server_time_url() -> &'static str {
        CaptchaProtocol::GET_SERVER_TIME
    }
    fn get_captcha_url() -> &'static str {
        CaptchaProtocol::GET_CAPTCHA
    }
    fn check_captcha_url() -> &'static str {
        CaptchaProtocol::CHECK_CAPTCHA
    }
    fn my_sign_captcha_utils_url() -> &'static str {
        CaptchaProtocol::MY_SIGN_CAPTCHA_UTILS
    }
    // 获取服务器时间。
    fn get_server_time(
        agent: &Agent,
        captcha_id: &str,
        time_stamp_mills: impl Display + Copy,
    ) -> Result<Response<Body>, AgentError> {
        let url = Self::get_server_time_url();
        let url =
            format!("{url}?callback={CALLBACK_NAME}&captchaId={captcha_id}&_={time_stamp_mills}");
        Ok(agent.get(&url).call()?)
    }
    // 获取滑块。
    fn get_captcha(
        agent: &Agent,
        captcha_type: &str,
        captcha_id: &str,
        (captcha_key, tmp_token): (&str, &str),
        iv: &str,
        time_stamp_mills: impl Display + Copy,
        referer: &str,
    ) -> Result<Response<Body>, AgentError> {
        let url = Self::get_captcha_url();
        let referer =
            percent_encoding::utf8_percent_encode(referer, percent_encoding::NON_ALPHANUMERIC)
                .to_string();
        let url = format!(
            "{url}?{callback}&{id}&{key}&{token}&{iv}&{type_}&{version}&{referer_}&_={time_stamp_mills}",
            callback = format_args!("callback={}", CALLBACK_NAME),
            id = format_args!("captchaId={}", captcha_id),
            key = format_args!("captchaKey={}", captcha_key),
            token = format_args!("token={}", tmp_token),
            iv = format_args!("iv={}", iv),
            type_ = format_args!("type={}", captcha_type),
            version = VERSION_PARAM,
            referer_ = format_args!("referer={}", referer),
        );
        Ok(agent.get(&url).header("Referer", &referer).call()?)
    }

    // 滑块验证。
    fn check_captcha(
        agent: &Agent,
        captcha_type: &str,
        captcha_id: &str,
        text_click_arr: impl Display,
        token: &str,
        iv: &str,
        time_stamp_mills: impl Display + Copy,
    ) -> Result<Response<Body>, AgentError> {
        let url = Self::check_captcha_url();
        let url = format!(
            "{url}?{}&{}&{}&{}&{}&{}&{}&{}&{}&_={time_stamp_mills}",
            format_args!("callback={CALLBACK_NAME}",),
            format_args!("captchaId={}", captcha_id),
            format_args!("token={}", token),
            format_args!("textClickArr={}", text_click_arr),
            format_args!("iv={}", iv),
            format_args!("type={}", captcha_type),
            "coordinate=%5B%5D",
            VERSION_PARAM,
            // WEB = 10
            // ANDROID = 20
            // IOS = 30
            // MINIPROGRAM = 40
            "runEnv=20",
        );
        let get = agent
            .get(&url)
            .header("Referer", "https://mobilelearn.chaoxing.com");
        Ok(get.call()?)
    }

    fn my_sign_captcha_utils(client: &Agent) -> Result<Response<Body>, AgentError> {
        let url = Self::my_sign_captcha_utils_url();
        debug!("{url}");
        Ok(client.get(&url.to_string()).call()?)
    }
}
pub struct CaptchaProtocol;
impl CaptchaProtocol {
    /// 获取滑块。
    pub const GET_CAPTCHA: &'static str =
        "https://captcha.chaoxing.com/captcha/get/verification/image";
    /// 滑块验证。
    pub const CHECK_CAPTCHA: &'static str =
        "https://captcha.chaoxing.com/captcha/check/verification/result";
    /// 获取服务器时间。
    pub const GET_SERVER_TIME: &'static str = "https://captcha.chaoxing.com/captcha/get/conf";
    pub const MY_SIGN_CAPTCHA_UTILS: &'static str =
        "https://mobilelearn.chaoxing.com/front/mobile/sign/js/mySignCaptchaUtils.js";
}
impl CaptchaProtocolTrait for CaptchaProtocol {}
