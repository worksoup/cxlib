use crate::utils::CaptchaType;
use cxlib_protocol::ProtocolItem;
use cxlib_utils::url_encode;
use log::debug;
use std::fmt::Display;
use ureq::Agent;

// Doesn't matter.
pub(crate) static CALLBACK_NAME: &str = "jQuery_114514_1919810";

// 获取服务器时间。
pub fn get_server_time(
    agent: &Agent,
    captcha_id: &str,
    time_stamp_mills: impl Display + Copy,
) -> Result<ureq::Response, Box<ureq::Error>> {
    let url = format!(
        "{}?callback={CALLBACK_NAME}&captchaId={captcha_id}&_={time_stamp_mills}",
        ProtocolItem::GetServerTime,
    );
    Ok(agent.get(&url).call()?)
}
static VERSION_PARAM: &str = "version=1.1.20";
// 获取滑块。
pub fn get_captcha(
    agent: &Agent,
    captcha_type: &CaptchaType,
    captcha_id: &str,
    (captcha_key, tmp_token): (&str, &str),
    iv: &str,
    time_stamp_mills: impl Display + Copy,
    referer: &str,
) -> Result<ureq::Response, Box<ureq::Error>> {
    let url = format!(
        "{}?{callback}&{id}&{key}&{token}&{iv}&{type_}&{version}&{referer_}&_={time_stamp_mills}",
        ProtocolItem::GetCaptcha,
        callback = format_args!("callback={}", CALLBACK_NAME),
        id = format_args!("captchaId={}", captcha_id),
        key = format_args!("captchaKey={}", captcha_key),
        token = format_args!("token={}", tmp_token),
        iv = format_args!("iv={}", iv),
        type_ = format_args!("type={}", captcha_type),
        version = VERSION_PARAM,
        referer_ = format_args!("referer={}", url_encode(referer)),
    );
    Ok(agent.get(&url).set("Referer", referer).call()?)
}

// 滑块验证。
pub fn check_captcha(
    agent: &Agent,
    captcha_type: &CaptchaType,
    captcha_id: &str,
    text_click_arr: impl Display,
    token: &str,
    iv: &str,
    time_stamp_mills: impl Display + Copy,
) -> Result<ureq::Response, Box<ureq::Error>> {
    let url = format!(
        "{}?{}&{}&{}&{}&{}&{}&{}&{}&{}&_={time_stamp_mills}",
        ProtocolItem::CheckCaptcha,
        format_args!("callback={CALLBACK_NAME}",),
        format_args!("captchaId={}", captcha_id),
        format_args!("token={}", token),
        format_args!("textClickArr={}", text_click_arr),
        format_args!("iv={}", iv),
        format_args!("type={}", captcha_type),
        "coordinate=%5B%5D",
        VERSION_PARAM,
        "runEnv=10",
    );
    let get = agent
        .get(&url)
        .set("Referer", "https://mobilelearn.chaoxing.com");
    Ok(get.call()?)
}

pub fn my_sign_captcha_utils(client: &Agent) -> Result<ureq::Response, Box<ureq::Error>> {
    let url = ProtocolItem::MySignCaptchaUtils;
    debug!("{url}");
    Ok(client.get(&url.to_string()).call()?)
}
