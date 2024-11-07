use cxsign_protocol::ProtocolEnum;
use log::debug;
use std::fmt::Display;
use ureq::{Agent, Response};

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
        ProtocolEnum::GetServerTime,
    );
    Ok(agent.get(&url).call()?)
}

// 获取滑块。
pub fn get_captcha(
    agent: &Agent,
    captcha_id: &str,
    captcha_key: &str,
    tmp_token: &str,
    time_stamp_mills: impl Display + Copy,
) -> Result<ureq::Response, Box<ureq::Error>> {
    let url = format!(
        "{}?{}&{}&{}&{}&{}&{}&{}&_={time_stamp_mills}",
        ProtocolEnum::GetCaptcha,
        format_args!("callback={}", CALLBACK_NAME),
        format_args!("captchaId={}", captcha_id),
        format_args!("captchaKey={}", captcha_key),
        format_args!("token={}", tmp_token),
        "type=slide",
        "version=1.1.16",
        "referer=https%3A%2F%2Fmobilelearn.chaoxing.com",
    );
    Ok(agent.get(&url).call()?)
}

// 滑块验证。
pub fn check_captcha(
    agent: &Agent,
    captcha_id: &str,
    x: impl Display + Copy,
    token: &str,
    time_stamp_mills: impl Display + Copy,
) -> Result<ureq::Response, Box<ureq::Error>> {
    let url = format!(
        "{}?{}&{}&{}&{}&{}&{}&{}&{}&_={time_stamp_mills}",
        ProtocolEnum::CheckCaptcha,
        format_args!("callback={CALLBACK_NAME}",),
        format_args!("captchaId={}", captcha_id),
        format_args!("token={}", token),
        format_args!("textClickArr=%5B%7B%22x%22%3A{}%7D%5D", x),
        "type=slide",
        "coordinate=%5B%5D",
        "version=1.1.16",
        "runEnv=10",
    );
    let get = agent
        .get(&url)
        .set("Referer", "https://mobilelearn.chaoxing.com");
    Ok(get.call()?)
}

pub fn my_sign_captcha_utils(client: &Agent) -> Result<Response, Box<ureq::Error>> {
    let url = ProtocolEnum::MySignCaptchaUtils;
    debug!("{url}");
    Ok(client.get(&url).call()?)
}
