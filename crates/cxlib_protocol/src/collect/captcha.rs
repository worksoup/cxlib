use cxlib_error::AgentError;
use cxlib_error_utils::CxlibResultUtils;
use log::debug;
use std::fmt::Display;
use ureq::Agent;

pub use types::*;

use crate::{ProtocolError, utils::trim_response_to_json};
// Doesn't matter.
pub static CALLBACK_NAME: &str = "cx_captcha_function";
static VERSION_PARAM: &str = "version=1.1.20";
pub static CAPTCHA_ID: &str = "Qt9FIw9o4pwRjOyqM6yizZBh682qN2TU";

mod types {
    /// 该文档为AI生成。包含令牌和验证数据的结构
    #[derive(Debug, serde::Deserialize)]
    pub struct VerificationDataWithToken {
        /// 该文档为AI生成。验证令牌
        pub token: String,
        /// 该文档为AI生成。验证码数据(JSON格式)
        #[serde(rename = "imageVerificationVo")]
        pub data: serde_json::Value,
    }
    /// 该文档为AI生成。验证结果结构体
    #[derive(serde::Deserialize, Debug)]
    pub struct ValidateResult {
        /// 该文档为AI生成。额外的验证数据
        #[serde(rename = "extraData")]
        extra_data: Option<String>,
    }
    impl ValidateResult {
        /// 该文档为AI生成。从验证结果中提取验证信息
        #[inline]
        pub fn get_validate_info(&self) -> Option<String> {
            #[derive(serde::Deserialize)]
            struct Tmp {
                validate: String,
            }
            self.extra_data.as_ref().map(|s| {
                let Tmp { validate } = serde_json::from_str(s).unwrap();
                validate
            })
        }
    }
    pub type CaptchaId = String;
}

pub trait CaptchaProtocolTrait {
    #[inline]
    fn get_server_time_url() -> &'static str {
        CaptchaProtocol::GET_SERVER_TIME
    }
    #[inline]
    fn get_captcha_url() -> &'static str {
        CaptchaProtocol::GET_CAPTCHA
    }
    #[inline]
    fn check_captcha_url() -> &'static str {
        CaptchaProtocol::CHECK_CAPTCHA
    }
    #[inline]
    fn my_sign_captcha_utils_url() -> &'static str {
        CaptchaProtocol::MY_SIGN_CAPTCHA_UTILS
    }
    // 获取服务器时间。
    #[inline]
    fn get_server_time_mills(
        agent: &Agent,
        captcha_id: &str,
        time_stamp_mills: impl Display + Copy,
    ) -> Result<u128, AgentError> {
        let url = Self::get_server_time_url();
        let url =
            format!("{url}?callback={CALLBACK_NAME}&captchaId={captcha_id}&_={time_stamp_mills}");
        #[derive(serde::Deserialize)]
        struct Tmp {
            t: u128,
        }
        let Tmp { t } =
            crate::utils::trim_response_to_json(agent.get(&url).call()?.into_body().into_reader())
                .log_unwrap();
        Ok(t)
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
    ) -> Result<VerificationDataWithToken, AgentError> {
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
        Ok(trim_response_to_json(
            agent
                .get(&url)
                .header("Referer", &referer)
                .call()?
                .into_body()
                .into_reader(),
        )
        .log_unwrap())
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
    ) -> Result<ValidateResult, AgentError> {
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
        Ok(trim_response_to_json(get.call()?.into_body().into_reader()).log_unwrap())
    }

    fn get_captcha_id(agent: &Agent) -> Result<CaptchaId, ProtocolError> {
        let url = Self::my_sign_captcha_utils_url();
        debug!("{url}");
        let r = agent.get(&url.to_string()).call()?;
        let js = r.into_body().read_to_string().log_unwrap();
        js.find("captchaId: '")
            .map(|start_of_captcha_id| {
                debug!("start_of_captcha_id: {start_of_captcha_id}");
                let id = &js[start_of_captcha_id + 12..start_of_captcha_id + 12 + 32];
                debug!("captcha_id: {id}");
                id.to_string()
            })
            .ok_or_else(|| {
                ProtocolError::DataParseError("查找 CaptchaId 失败，可能是 API 更新。".to_owned())
            })
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
