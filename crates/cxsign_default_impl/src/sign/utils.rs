use crate::sign::CaptchaId;
use cxsign_activity::RawSign;
use cxsign_captcha::protocol::CAPTCHA_ID;
use cxsign_sign::{SignResult, SignTrait};
use cxsign_types::{Location, LocationWithRange};
use cxsign_user::Session;
use log::{debug, warn};
use ureq::Response;

pub fn secondary_verification(
    agent: &ureq::Agent,
    url: String,
    captcha_id: &Option<CaptchaId>,
) -> Result<SignResult, cxsign_error::Error> {
    let captcha_id = if let Some(captcha_id) = captcha_id {
        captcha_id
    } else {
        warn!("未找到滑块 ID, 使用内建值。");
        CAPTCHA_ID
    };
    let url_param = cxsign_captcha::utils::captcha_solver(agent, captcha_id)?;
    let r = {
        let url = url + "&validate=" + &url_param;
        let r = ureq_get(agent, &url)?;
        RawSign::guess_sign_result_by_text(&r.into_string().unwrap())
    };
    Ok(r)
}

pub fn sign_unchecked_with_location<T: SignTrait>(
    url_getter: impl Fn(&Location) -> String,
    location: &Location,
    preset_location: &Option<LocationWithRange>,
    captcha_id: Option<CaptchaId>,
    session: &Session,
) -> Result<SignResult, cxsign_error::Error> {
    let mut locations = Vec::new();
    let addr = location.get_addr();
    locations.push(location.clone());
    if let Some(location) = preset_location {
        let mut location = location.to_shifted_location();
        if !addr.is_empty() {
            location.set_addr(addr);
        }
        locations.push(location);
    }
    if locations.is_empty() {
        return Ok(SignResult::Fail {
            msg: "没有可供签到的位置！".to_string(),
        });
    }
    for location in locations {
        let url = url_getter(&location);
        let r = ureq_get(session, &url)?;
        match T::guess_sign_result_by_text(&r.into_string().unwrap()) {
            SignResult::Susses => return Ok(SignResult::Susses),
            SignResult::Fail { msg } => {
                if msg.starts_with("validate") {
                    // 这里假设了二次验证只有在“签到成功”的情况下出现。
                    let url = if msg.len() > 9 {
                        let enc2 = &msg[9..msg.len()];
                        debug!("enc2: {enc2:?}");
                        url + "&enc2=" + enc2
                    } else {
                        url
                    };
                    return secondary_verification(session, url, &captcha_id);
                } else if msg.contains("位置") || msg.contains("Location") || msg.contains("范围")
                {
                    continue;
                } else {
                    return Ok(SignResult::Fail { msg });
                }
            }
        };
    }
    warn!("BUG: 请保留现场联系开发者处理。");
    Ok(SignResult::Fail {
        msg: "所有位置均不可用。".to_string(),
    })
}
pub fn ureq_get(agent: &ureq::Agent, url: &str) -> Result<Response, Box<ureq::Error>> {
    Ok(agent.get(url).call()?)
}
