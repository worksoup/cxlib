//! 该文档为AI生成。验证码处理模块
//!
//! 提供验证码获取、解析和验证的完整流程支持，
//! 支持多种验证码类型，包括滑块、文字点选、图片旋转等。

use crate::{
    CaptchaError, VerificationInfoTrait,
    hash::{encode, hash, uuid},
    utils::get_now_timestamp_mills,
};
use cxlib_error::AgentError;
use cxlib_error_utils::{CxlibResultUtils, MaybeFatalError};
use cxlib_protocol::collect::{CaptchaProtocolTrait, VerificationDataWithToken};
use log::{debug, warn};
use serde::de::DeserializeOwned;
use ureq::Agent;

/// 该文档为AI生成。获取验证码的结果结构体
#[derive(Debug)]
pub struct GetCaptchaResult {
    /// 该文档为AI生成。验证码的初始化向量(IV)
    pub iv: String,
    /// 该文档为AI生成。包含令牌和验证数据的结构
    pub data: VerificationDataWithToken,
}
/// 该文档为AI生成。验证码解决器特性
///
/// 定义了验证码处理的通用接口，包括生成密钥、获取验证码、验证验证码等
pub trait CaptchaSolverTrait {
    /// 该文档为AI生成。解决验证码的核心方法
    fn solver(
        agent: &Agent,
        image: serde_json::Value,
        referer: &str,
    ) -> Result<String, CaptchaError>;

    /// 该文档为AI生成。生成验证码所需的密钥
    fn generate_secrets(captcha_id: &str, server_time_stamp_mills: u128) -> (String, String);

    /// 该文档为AI生成。生成初始化向量(IV)
    fn generate_iv(captcha_id: &str) -> String;

    /// 该文档为AI生成。获取验证码数据
    fn get_captcha<CaptchaProtocol: CaptchaProtocolTrait>(
        agent: &Agent,
        captcha_id: &str,
        server_time_mills: u128,
        referer: &str,
    ) -> Result<GetCaptchaResult, AgentError>;

    /// 该文档为AI生成。验证验证码结果
    fn check_captcha<CaptchaProtocol: CaptchaProtocolTrait>(
        agent: &Agent,
        captcha_id_iv_token: (&str, &str, &str),
        text_click_arr: &str,
        server_time_mills: u128,
    ) -> Result<String, CaptchaError>;

    /// 该文档为AI生成。完整的验证码解决流程
    fn solve_captcha<CaptchaProtocol: CaptchaProtocolTrait>(
        agent: &Agent,
        captcha_id: &str,
        referer: &str,
    ) -> Result<String, CaptchaError>;
}

/// 该文档为AI生成。为所有实现VerificationInfoTrait的类型提供默认实现
impl<T> CaptchaSolverTrait for T
where
    T: VerificationInfoTrait + DeserializeOwned + 'static,
{
    /// 该文档为AI生成。解决验证码的核心方法实现
    fn solver(
        agent: &Agent,
        image: serde_json::Value,
        referer: &str,
    ) -> Result<String, CaptchaError> {
        let self_: Self = serde_json::from_value(image).log_unwrap();
        self_.solve(agent, referer)
    }

    /// 该文档为AI生成。生成验证码所需的密钥实现
    fn generate_secrets(captcha_id: &str, server_time_stamp_mills: u128) -> (String, String) {
        let server_time_str = server_time_stamp_mills.to_string();
        let captcha_key = encode(hash(&(server_time_str.clone() + &uuid())));
        // "%3A" 即英文冒号的转义。
        let tmp_token = encode(hash(
            &(server_time_str + captcha_id + Self::captcha_type() + &captcha_key),
        )) + "%3A"
            + (server_time_stamp_mills + 300000_u128).to_string().as_str();
        (captcha_key, tmp_token)
    }

    /// 该文档为AI生成。生成初始化向量(IV)实现
    fn generate_iv(captcha_id: &str) -> String {
        let iv_uuid = uuid();
        encode(hash(
            &(captcha_id.to_owned()
                + Self::captcha_type()
                + get_now_timestamp_mills().to_string().as_str()
                + &iv_uuid),
        ))
    }

    /// 该文档为AI生成。获取验证码数据实现
    fn get_captcha<CaptchaProtocol: CaptchaProtocolTrait>(
        agent: &Agent,
        captcha_id: &str,
        server_time_mills: u128,
        referer: &str,
    ) -> Result<GetCaptchaResult, AgentError> {
        let (captcha_key, tmp_token) = Self::generate_secrets(captcha_id, server_time_mills);
        let iv = Self::generate_iv(captcha_id);
        let data = CaptchaProtocol::get_captcha(
            agent,
            Self::captcha_type(),
            captcha_id,
            (&captcha_key, &tmp_token),
            &iv,
            server_time_mills + 1,
            referer,
        )?;
        Ok(GetCaptchaResult { iv, data })
    }

    /// 该文档为AI生成。验证验证码结果实现
    fn check_captcha<CaptchaProtocol: CaptchaProtocolTrait>(
        agent: &Agent,
        (captcha_id, iv, token): (&str, &str, &str),
        text_click_arr: &str,
        server_time_mills: u128,
    ) -> Result<String, CaptchaError> {
        let v = CaptchaProtocol::check_captcha(
            agent,
            Self::captcha_type(),
            captcha_id,
            text_click_arr,
            token,
            iv,
            server_time_mills + 2,
        )?;
        debug!("验证结果：{v:?}");
        v.get_validate_info()
            .ok_or_else(|| CaptchaError::VerifyFailed)
    }

    /// 该文档为AI生成。完整的验证码解决流程实现
    ///
    /// 包含获取验证码、解决验证码和验证结果的全过程
    fn solve_captcha<CaptchaProtocol: CaptchaProtocolTrait>(
        agent: &Agent,
        captcha_id: &str,
        referer: &str,
    ) -> Result<String, CaptchaError> {
        let local_time = get_now_timestamp_mills();
        let server_time = CaptchaProtocol::get_server_time_mills(agent, captcha_id, local_time)?;

        // 事不过三。
        for i in 0..3 {
            match Self::get_captcha::<CaptchaProtocol>(agent, captcha_id, server_time + i, referer)
                .map_err(CaptchaError::from)
                .and_then(
                    |GetCaptchaResult {
                         iv,
                         data: VerificationDataWithToken { token, data },
                     }| {
                        Self::solver(agent, data, referer).and_then(|text_click_arr| {
                            Self::check_captcha::<CaptchaProtocol>(
                                agent,
                                (captcha_id, &iv, &token),
                                &text_click_arr,
                                server_time + i,
                            )
                        })
                    },
                ) {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if e.is_fatal() {
                        return Err(e);
                    } else {
                        warn!("验证码验证失败：{e}，即将重试。");
                    }
                }
            }
        }
        Err(CaptchaError::VerifyFailed)
    }
}

/// 该文档为AI生成。测试模块
#[cfg(test)]
mod tests {
    use crate::{
        CaptchaSolverTrait, RotateImages, SlideImages, VerificationInfoTrait,
        hash::{encode, hash},
        utils::get_now_timestamp_mills,
    };
    use cxlib_protocol::collect::{CAPTCHA_ID, CaptchaProtocol, CaptchaProtocolTrait};
    use serde::de::DeserializeOwned;

    const REFERER: &str = "https%3A%2F%2Fmobilelearn.chaoxing.com";

    /// 该文档为AI生成。测试自动解决验证码功能
    #[test]
    fn auto_solve_captcha_test() {
        let agent = ureq::Agent::new_with_defaults();
        let r = <RotateImages>::solve_captcha::<CaptchaProtocol>(&agent, CAPTCHA_ID, REFERER);
        println!("{r:?}");
    }

    /// 该文档为AI生成。测试生成验证码密钥功能
    #[test]
    fn generate_captcha_key() {
        fn assert_eq_with_real_value<T>(
            real_value: &str,
            server_time: u128,
            captcha_id: &str,
            captcha_key: &str,
        ) where
            T: VerificationInfoTrait + DeserializeOwned + 'static,
        {
            let tmp_token = encode(hash(
                &(server_time.to_string() + captcha_id + T::captcha_type() + captcha_key),
            )) + "%3A"
                + (server_time + 300000_u128).to_string().as_str();
            assert_eq!(real_value, tmp_token);
        }
        assert_eq_with_real_value::<SlideImages>(
            "21d29919dc55f9a25b25a9aec531682e%3A1733129174649",
            1733128874649,
            CAPTCHA_ID,
            "0062a52fa1d93307b2bc503883986cf9",
        )
    }

    /// 该文档为AI生成。测试获取验证码功能
    #[test]
    fn get_captcha_test() {
        fn get_captcha_<T>()
        where
            T: VerificationInfoTrait + DeserializeOwned + 'static,
        {
            let agent = ureq::Agent::new_with_defaults();
            let local_time = get_now_timestamp_mills();
            let captcha_id = CAPTCHA_ID;
            let server_time =
                CaptchaProtocol::get_server_time_mills(&agent, captcha_id, local_time).unwrap();
            let validate_info =
                T::get_captcha::<CaptchaProtocol>(&agent, captcha_id, server_time + 1, REFERER)
                    .unwrap();
            println!("{validate_info:?}");
        }
        get_captcha_::<RotateImages>();
        get_captcha_::<SlideImages>();
    }
}
