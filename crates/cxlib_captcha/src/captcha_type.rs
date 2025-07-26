//! 该文档为AI生成。验证码处理模块
//!
//! 提供验证码获取、解析和验证的完整流程支持，
//! 支持多种验证码类型，包括滑块、文字点选、图片旋转等。

use std::{fmt::Display, str::FromStr};

use crate::{
    CaptchaError, IconClickImage, ObstacleImage, RotateImages, SlideImages, TextClickInfo,
    VerificationInfoTrait,
    hash::{encode, hash, uuid},
    utils::{get_now_timestamp_mills, get_server_time, trim_response_to_json},
};
use cxlib_error::AgentError;
use cxlib_error_utils::{CxlibResultUtils, MaybeFatalError};
use cxlib_protocol::collect::CaptchaProtocolTrait;
use log::{debug, warn};
use serde::{Deserialize, de::DeserializeOwned};
use ureq::Agent;

/// 该文档为AI生成。获取验证码的结果结构体
#[derive(Debug)]
pub struct GetCaptchaResult {
    /// 该文档为AI生成。验证码的初始化向量(IV)
    pub iv: String,
    /// 该文档为AI生成。包含令牌和验证数据的结构
    pub data: VerificationDataWithToken,
}

/// 该文档为AI生成。包含令牌和验证数据的结构
#[derive(Debug, Deserialize)]
pub struct VerificationDataWithToken {
    /// 该文档为AI生成。验证令牌
    pub token: String,
    /// 该文档为AI生成。验证码数据(JSON格式)
    #[serde(rename = "imageVerificationVo")]
    pub data: serde_json::Value,
}

/// 该文档为AI生成。验证结果结构体
#[derive(Deserialize, Debug)]
pub struct ValidateResult {
    /// 该文档为AI生成。额外的验证数据
    #[serde(rename = "extraData")]
    extra_data: Option<String>,
}

impl ValidateResult {
    /// 该文档为AI生成。从验证结果中提取验证信息
    pub fn get_validate_info(&self) -> Result<String, CaptchaError> {
        #[derive(Deserialize)]
        struct Tmp {
            validate: String,
        }
        self.extra_data
            .as_ref()
            .map(|s| {
                debug!("验证结果数据: {s}");
                let Tmp { validate } = serde_json::from_str(s).unwrap();
                validate
            })
            .ok_or_else(|| CaptchaError::VerifyFailed)
    }
}

/// 验证码类型枚举
///
/// # [`CaptchaType`]
/// 支持多种验证码类型，每种类型对应不同的验证机制：
/// - `Slide`: 滑块验证码
/// - `TextClick`: 文字点选验证码
/// - `Rotate`: 图片旋转验证码
/// - `IconClick`: 图标点选验证码
/// - `Obstacle`: 单图标点选验证码
///   
/// 目前只有 [`CaptchaType::Slide`] 与 [`CaptchaType::Rotate`] 类型支持良好.
///
/// TODO: fix doc.
/// 如需自行支持，请为该类型 [实现 `Solver`](CaptchaType::init_solver).
///
/// 若需自行处理图片下载等步骤，参见 [`CaptchaType::set_verification_info_type`].
/// 该函数可以替换掉默认的验证信息类型。
#[derive(Debug, Clone)]
pub enum CaptchaType {
    /// ## 滑块验证码
    /// 拖动滑块至合适位置，完成验证。
    ///
    /// 对应的验证信息类型为 [`SlideImages`],
    /// 如需自定义 `Solver`, 请参考其文档。
    Slide,
    /// ## 文字点选验证码
    /// 按照提示依次点击三个汉字，完成验证。
    ///
    /// 对应的验证信息类型为 [`TextClickInfo`],
    /// 请参考其文档初始化 `Solver`.
    TextClick,
    /// ## 图片旋转验证码
    /// 将图片旋转至合适角度，完成验证。
    ///
    /// 对应的验证信息类型为 [`RotateImages`],
    /// 请参考其文档初始化 `Solver`.
    Rotate,
    /// ## 图标点选验证码
    /// 按照提示依次点击三个图标，完成验证。
    ///
    /// 对应的验证信息类型为 [`IconClickImage`],
    /// 请参考其文档初始化 `Solver`.
    IconClick,
    /// ## 单图标点选验证码
    /// 按照提示点击单个图标，完成验证。
    ///
    /// 对应的验证信息类型为 [`ObstacleImage`],
    /// 请参考其文档初始化 `Solver`.
    Obstacle,
}

impl CaptchaType {
    /// 该文档为AI生成。默认验证码类型(旋转验证码)
    const DEFAULT: CaptchaType = CaptchaType::Rotate;
}

impl Default for CaptchaType {
    /// 该文档为AI生成。获取默认验证码类型
    #[inline]
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl FromStr for CaptchaType {
    type Err = CaptchaError;

    /// 该文档为AI生成。从字符串解析验证码类型
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "slide" | "Slide" | "SLIDE" => CaptchaType::Slide,
            "iconclick" | "IconClick" | "ICONCLICK" | "icon_click" | "ICON_CLICK" => {
                CaptchaType::IconClick
            }
            "textclick" | "TextClick" | "TEXTCLICK" | "text_click" | "TEXT_CLICK" => {
                CaptchaType::TextClick
            }
            "obstacle" | "Obstacle" | "OBSTACLE" => CaptchaType::Obstacle,
            "rotate" | "Rotate" | "ROTATE" => CaptchaType::Rotate,
            _ => Err(CaptchaError::UnsupportedType)?,
        })
    }
}

impl Display for CaptchaType {
    /// 该文档为AI生成。将验证码类型转换为字符串表示
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl AsRef<str> for CaptchaType {
    /// 该文档为AI生成。获取验证码类型的字符串引用
    #[inline]
    fn as_ref(&self) -> &str {
        match self {
            CaptchaType::Slide => "slide",
            CaptchaType::TextClick => "textclick",
            CaptchaType::Rotate => "rotate",
            CaptchaType::IconClick => "iconclick",
            CaptchaType::Obstacle => "obstacle",
        }
    }
}

/// 该文档为AI生成。验证码解决器函数类型
pub type CaptchaSolver = fn(&Agent, &str, &str) -> Result<String, CaptchaError>;

impl CaptchaType {
    /// 该文档为AI生成。获取验证码类型的解决器函数
    ///
    /// 根据验证码类型返回对应的解决函数
    pub fn solver<CaptchaProtocol: CaptchaProtocolTrait>(&self) -> CaptchaSolver {
        match self {
            CaptchaType::Slide => SlideImages::solve_captcha::<CaptchaProtocol>,
            CaptchaType::TextClick => TextClickInfo::solve_captcha::<CaptchaProtocol>,
            CaptchaType::Rotate => RotateImages::solve_captcha::<CaptchaProtocol>,
            CaptchaType::IconClick => IconClickImage::solve_captcha::<CaptchaProtocol>,
            CaptchaType::Obstacle => ObstacleImage::solve_captcha::<CaptchaProtocol>,
        }
    }
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
        let r = CaptchaProtocol::get_captcha(
            agent,
            Self::captcha_type(),
            captcha_id,
            (&captcha_key, &tmp_token),
            &iv,
            server_time_mills + 1,
            referer,
        )?;
        let r_data = trim_response_to_json(
            &r.into_body()
                .read_to_string()
                .expect("CaptchaResponse into String failed."),
        )
        .expect("Failed trim_response_to_json");
        Ok(GetCaptchaResult { iv, data: r_data })
    }

    /// 该文档为AI生成。验证验证码结果实现
    fn check_captcha<CaptchaProtocol: CaptchaProtocolTrait>(
        agent: &Agent,
        (captcha_id, iv, token): (&str, &str, &str),
        text_click_arr: &str,
        server_time_mills: u128,
    ) -> Result<String, CaptchaError> {
        let r = CaptchaProtocol::check_captcha(
            agent,
            Self::captcha_type(),
            captcha_id,
            text_click_arr,
            token,
            iv,
            server_time_mills + 2,
        )?;
        let v: ValidateResult =
            trim_response_to_json(&r.into_body().read_to_string().log_unwrap()).log_unwrap();
        debug!("验证结果：{v:?}");
        v.get_validate_info()
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
        let server_time = get_server_time::<CaptchaProtocol>(agent, captcha_id, local_time)?;

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
        CaptchaSolverTrait, IconClickImage, ObstacleImage, RotateImages, SlideImages,
        TextClickInfo, VerificationInfoTrait,
        hash::{encode, hash},
        utils::{get_now_timestamp_mills, get_server_time},
    };
    use cxlib_protocol::collect::{CAPTCHA_ID, CaptchaProtocol};
    use serde::de::DeserializeOwned;

    const REFERER: &str = "https%3A%2F%2Fmobilelearn.chaoxing.com";

    /// 该文档为AI生成。测试自动解决验证码功能
    #[test]
    fn auto_solve_captcha_test() {
        let agent = ureq::Agent::new_with_defaults();
        let r = RotateImages::solve_captcha::<CaptchaProtocol>(&agent, CAPTCHA_ID, REFERER);
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
        assert_eq_with_real_value::<IconClickImage>(
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
                get_server_time::<CaptchaProtocol>(&agent, captcha_id, local_time).unwrap();
            let validate_info =
                T::get_captcha::<CaptchaProtocol>(&agent, captcha_id, server_time + 1, REFERER)
                    .unwrap();
            println!("{validate_info:?}");
        }
        get_captcha_::<IconClickImage>();
        get_captcha_::<ObstacleImage>();
        get_captcha_::<RotateImages>();
        get_captcha_::<SlideImages>();
        get_captcha_::<TextClickInfo>();
    }
}
