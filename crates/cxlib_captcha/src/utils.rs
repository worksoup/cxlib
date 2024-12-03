//!# 混淆代码阅读指南
//! CX 获取验证码的代码被混淆过，浏览器打开预签到的地址即可查看。
//!
//! 代码在 `load.min.js` 里，可以先扔到 [jstillery](https://mindedsecurity.github.io/jstillery/)　里面
//! 格式化一下，反混淆其实没多大作用。
//!
//! 接着在网络请求中查看 `captcha/get/verification/image` 请求的栈跟踪，依次查看调用函数。
//! 其中有两个函数需要注意：
//! 1. 一个“长函数”，应该是 hash 函数，特点是在返回值为函数，且返回语句下方定义了不少“短函数”。
//! > 猜测应该是一个变种的 md5.
//! 2. 一个“短函数”，猜测应该是 uuid 函数。特点是内部是一个循环。然后为一个数组循环赋值。最后返回这个数组。
//!
//! ## 已知参数计算方式：
//! - `captchaId`: 常值。
//! - `type`: 验证码类型，注意为全小写格式，如 `IconClick => "iconclick"`:
//!```
//! pub enum CaptchaType{
//!     Slide,
//!     TextClick,
//!     Rotate,
//!     IconClick,
//!     Obstacle
//! }
//! ```
//! - `version`: 可以看作常值。
//! - `captchaKey`: hash("{serverTime}" + rand_uuid())
//! - `token`: hash("{serverTime}" + captchaId + type + captchaKey) + ':' + "{serverTime + 300000}"
//! - `referer`: 即预签到地址。
//! - `iv`: hash(captchaId + type + Date.now() + rand_uuid())
//!
//! ## 整体流程：
//! 1. getServerTime;
//! ``` json
//! {
//!     "t": 12903871908,
//!     "captchaId": "Qt9FIw9o4pwRjOyqM6yizZBh682qN2TU"
//! }
//! ```
//! 2. getCaptchaImage;
//!    这个也会返回一个 `token`, 但实际上与请求中的不是一个，这个验证结果时用。
//! 3. checkVerificationResult;
//!    请求参数有刚返回的 `token`, 其他可以在网络请求里看到。当然也包含刚刚计算的 iv.

use crate::{
    hash::{encode, hash, uuid},
    protocol::{check_captcha, get_captcha},
    CaptchaId, IconClickImage, ObstacleImage, RotateImages, SlideImages, TextClickInfo,
    VerificationInfoTrait, DEFAULT_CAPTCHA_TYPE,
};
use cxlib_error::UnwrapOrLogPanic;
use log::{debug, warn};
use onceinit::{OnceInitError, StaticDefault};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::fmt::{Display, Formatter};
use ureq::{serde_json, Agent, Error};

pub fn get_now_timestamp_mills() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("系统时间异常。")
        .as_millis()
}
pub fn get_server_time(
    agent: &Agent,
    captcha_id: &str,
    time_stamp_mills: impl Display + Copy,
) -> Result<u128, Box<Error>> {
    let r = crate::protocol::get_server_time(agent, captcha_id, time_stamp_mills)?;
    #[derive(Deserialize)]
    struct Tmp {
        t: u128,
    }
    let Tmp { t } =
        trim_response_to_json(r.into_string().unwrap_or_log_panic().as_str()).unwrap_or_log_panic();
    Ok(t)
}
pub fn trim_response_to_json<'a, T>(text: &'a str) -> Result<T, serde_json::Error>
where
    T: serde::de::Deserialize<'a>,
{
    let s = &text[crate::protocol::CALLBACK_NAME.len() + 1..text.len() - 1];
    debug!("{s}");
    serde_json::from_str(s)
}
#[derive(Debug)]
pub struct GetCaptchaResult {
    pub iv: String,
    pub data: VerificationDataWithToken,
}
#[derive(Debug, Deserialize)]
pub struct VerificationDataWithToken {
    pub token: String,
    #[serde(rename = "imageVerificationVo")]
    pub data: serde_json::Value,
}
/// # [`CaptchaType`]
/// 验证码类型，目前只有 [`CaptchaType::Slide`] 类型支持良好，无需初始化 `Solver`.
/// 如需自行支持，请为该类型 [实现 `Solver`](VerificationInfoTrait::init_solver)
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
impl StaticDefault for CaptchaType {
    fn static_default() -> &'static Self {
        &CaptchaType::Slide
    }
}
impl Display for CaptchaType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}
impl AsRef<str> for CaptchaType {
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
impl CaptchaType {
    /// 将当前验证码类型设为全局默认。
    ///
    /// 注意，默认类型仅可设置一次。
    pub fn as_global_default(&self) -> Result<(), OnceInitError> {
        DEFAULT_CAPTCHA_TYPE.set_boxed_data(Box::new(self.clone()))
    }
    /// 将设置全局默认的验证码类型。
    ///
    /// 注意，默认类型仅可设置一次。
    pub fn set_global_default(self_: &CaptchaType) -> Result<(), OnceInitError> {
        self_.as_global_default()
    }
    pub fn generate_secrets(
        &self,
        captcha_id: &str,
        server_time_stamp_mills: u128,
    ) -> (String, String) {
        let server_time_str = server_time_stamp_mills.to_string();
        let captcha_key = encode(hash(&(server_time_str.clone() + &uuid())));
        // "%3A" 即英文冒号的转义。
        let tmp_token = encode(hash(
            &(server_time_str + captcha_id + self.as_ref() + &captcha_key),
        )) + "%3A"
            + (server_time_stamp_mills + 300000_u128).to_string().as_str();
        (captcha_key, tmp_token)
    }

    pub fn generate_iv(&self, captcha_id: &str) -> String {
        let iv_uuid = uuid();
        let iv = encode(hash(
            &(captcha_id.to_owned()
                + self.as_ref()
                + get_now_timestamp_mills().to_string().as_str()
                + &iv_uuid),
        ));
        iv
    }
    pub fn get_captcha(
        &self,
        agent: &Agent,
        captcha_id: &str,
        server_time_mills: u128,
        referer: &str,
    ) -> Result<GetCaptchaResult, Box<Error>> {
        let (captcha_key, tmp_token) = self.generate_secrets(captcha_id, server_time_mills);
        let iv = self.generate_iv(captcha_id);
        let r = get_captcha(
            agent,
            self,
            captcha_id,
            (&captcha_key, &tmp_token),
            &iv,
            server_time_mills + 1,
            referer,
        )?;
        let r_data = trim_response_to_json(
            &r.into_string()
                .expect("CaptchaResponse into String failed."),
        )
        .expect("Failed trim_response_to_json");
        Ok(GetCaptchaResult { iv, data: r_data })
    }
    fn solve_captcha_<
        I: 'static,
        O: 'static,
        T: VerificationInfoTrait<I, O> + DeserializeOwned + 'static,
    >(
        &self,
        agent: &Agent,
        captcha_id: &str,
        server_time_mills: u128,
        referer: &str,
    ) -> Result<ValidateResult, cxlib_error::Error> {
        let GetCaptchaResult {
            iv,
            data: VerificationDataWithToken { token, data },
        } = self.get_captcha(agent, captcha_id, server_time_mills, referer)?;
        let r = check_captcha(
            agent,
            self,
            captcha_id,
            &T::solver(agent, data, referer)?,
            &token,
            &iv,
            server_time_mills + 2,
        )?;
        let v: ValidateResult =
            trim_response_to_json(&r.into_string().unwrap_or_log_panic()).unwrap();
        debug!("滑块结果：{v:?}");
        Ok(v)
    }
    pub fn solve_captcha(
        &self,
        agent: &Agent,
        captcha_id: &str,
        referer: &str,
    ) -> Result<String, cxlib_error::Error> {
        let local_time = get_now_timestamp_mills();
        let server_time = get_server_time(agent, captcha_id, local_time)?;
        // 事不过三。
        for i in 0..3 {
            let validate_info = match self {
                CaptchaType::Slide => Self::solve_captcha_::<_, _, SlideImages>,
                CaptchaType::IconClick => Self::solve_captcha_::<_, _, IconClickImage>,
                CaptchaType::TextClick => Self::solve_captcha_::<_, _, TextClickInfo>,
                CaptchaType::Obstacle => Self::solve_captcha_::<_, _, ObstacleImage>,
                CaptchaType::Rotate => Self::solve_captcha_::<_, _, RotateImages>,
            }(self, agent, captcha_id, server_time + i, referer)?
            .get_validate_info();
            if validate_info.is_ok() {
                return validate_info;
            } else {
                warn!("滑块验证失败，即将重试。")
            }
        }
        Err(cxlib_error::Error::CaptchaError("验证码为空。".to_owned()))
    }
}

pub fn find_captcha(client: &Agent, presign_html: &str) -> Option<CaptchaId> {
    if let Some(start_of_captcha_id) = presign_html.find("captchaId: '") {
        let id = &presign_html[start_of_captcha_id + 12..start_of_captcha_id + 12 + 32];
        debug!("captcha_id: {id}");
        Some(id.to_string())
    } else {
        crate::protocol::my_sign_captcha_utils(client)
            .ok()
            .and_then(|r| {
                let js = r.into_string().unwrap();
                js.find("captchaId: '").map(|start_of_captcha_id| {
                    debug!("start_of_captcha_id: {start_of_captcha_id}");
                    let id = &js[start_of_captcha_id + 12..start_of_captcha_id + 12 + 32];
                    debug!("captcha_id: {id}");
                    id.to_string()
                })
            })
    }
}

#[derive(Deserialize, Debug)]
pub struct ValidateResult {
    #[serde(rename = "extraData")]
    extra_data: Option<String>,
}

impl ValidateResult {
    pub fn get_validate_info(&self) -> Result<String, cxlib_error::Error> {
        #[derive(Deserialize)]
        struct Tmp {
            validate: String,
        }
        self.extra_data
            .as_ref()
            .map(|s| {
                debug!("{s}");
                let Tmp { validate } = serde_json::from_str(s).unwrap();
                validate
            })
            .ok_or_else(|| cxlib_error::Error::CaptchaError("验证码为空。".to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use crate::hash::{encode, hash};
    use crate::utils::{get_now_timestamp_mills, get_server_time, CaptchaType};
    use cxlib_protocol::{ProtocolItem, ProtocolItemTrait};

    const REFERER: &str = "https%3A%2F%2Fmobilelearn.chaoxing.com";
    #[test]
    fn auto_solve_captcha_test() {
        let agent = ureq::Agent::new();
        let r =
            CaptchaType::Slide.solve_captcha(&agent, &ProtocolItem::CaptchaId.to_string(), REFERER);
        println!("{:?}", r);
    }
    #[test]
    fn generate_captcha_key() {
        fn assert_eq_with_real_value(
            real_value: &str,
            server_time: u128,
            captcha_id: &str,
            captcha_type: CaptchaType,
            captcha_key: &str,
        ) {
            let tmp_token = encode(hash(
                &(server_time.to_string() + captcha_id + captcha_type.as_ref() + captcha_key),
            )) + "%3A"
                + (server_time + 300000_u128).to_string().as_str();
            assert_eq!(real_value, tmp_token);
        }
        assert_eq_with_real_value(
            "21d29919dc55f9a25b25a9aec531682e%3A1733129174649",
            1733128874649,
            ProtocolItem::CaptchaId.get().as_ref(),
            CaptchaType::IconClick,
            "0062a52fa1d93307b2bc503883986cf9",
        )
    }
    #[test]
    fn get_captcha_test() {
        fn get_captcha_(captcha_type: CaptchaType) {
            let agent = ureq::Agent::new();
            let local_time = get_now_timestamp_mills();
            let captcha_id = ProtocolItem::CaptchaId.get();
            let server_time = get_server_time(&agent, captcha_id.as_ref(), local_time).unwrap();
            let validate_info = captcha_type
                .get_captcha(&agent, captcha_id.as_ref(), server_time + 1, REFERER)
                .unwrap();
            println!("{:?}", validate_info);
        }
        get_captcha_(CaptchaType::IconClick);
        get_captcha_(CaptchaType::Obstacle);
        get_captcha_(CaptchaType::Rotate);
        get_captcha_(CaptchaType::Slide);
        get_captcha_(CaptchaType::TextClick);
    }
}
