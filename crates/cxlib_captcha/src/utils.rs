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

use crate::CaptchaId;
use cxlib_error::{AgentError, CxlibResultUtils};
use cxlib_imageproc::image_from_bytes;
use cxlib_protocol::collect::captcha as protocol;
use cxlib_utils::ureq_get_bytes;
use image::DynamicImage;
use log::debug;
use serde::Deserialize;
use std::fmt::Display;
use ureq::{serde_json, Agent};

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
) -> Result<u128, AgentError> {
    let r = protocol::get_server_time(agent, captcha_id, time_stamp_mills)?;
    #[derive(Deserialize)]
    struct Tmp {
        t: u128,
    }
    let Tmp { t } =
        trim_response_to_json(r.into_string().log_unwrap().as_str()).log_unwrap();
    Ok(t)
}
pub fn trim_response_to_json<'a, T>(text: &'a str) -> Result<T, serde_json::Error>
where
    T: serde::de::Deserialize<'a>,
{
    let s = &text[protocol::CALLBACK_NAME.len() + 1..text.len() - 1];
    debug!("{s}");
    serde_json::from_str(s)
}
pub fn find_captcha(client: &Agent, presign_html: &str) -> Option<CaptchaId> {
    if let Some(start_of_captcha_id) = presign_html.find("captchaId: '") {
        let id = &presign_html[start_of_captcha_id + 12..start_of_captcha_id + 12 + 32];
        debug!("captcha_id: {id}");
        Some(id.to_string())
    } else {
        protocol::my_sign_captcha_utils(client).ok().and_then(|r| {
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
pub fn download_image(
    agent: &Agent,
    image_url: &str,
    referer: &str,
) -> Result<DynamicImage, AgentError> {
    Ok(image_from_bytes(ureq_get_bytes(agent, image_url, referer)?))
}
