use crate::{protocol, PreSignResult, SignResult, SignTrait};
use cxlib_activity::RawSign;
use cxlib_captcha::utils::find_captcha;
use cxlib_captcha::CaptchaId;
use cxlib_protocol::{ProtocolItem, ProtocolItemTrait};
use cxlib_types::{Dioption, LocationWithRange};
use cxlib_user::Session;
use log::{debug, trace, warn};
use std::ops::{Deref, DerefMut};
use ureq::{Agent, Response};

pub fn analysis_after_presign(
    active_id: &str,
    session: &Session,
    response_of_presign: ureq::Response,
) -> Result<PreSignResult, cxlib_error::Error> {
    let html = response_of_presign.into_string().unwrap();
    trace!("预签到请求结果：{html}");
    if let Some(start_of_statuscontent_h1) = html.find("id=\"statuscontent\"") {
        let html = &html[start_of_statuscontent_h1 + 19..];
        let end_of_statuscontent_h1 = html.find("</").unwrap();
        let content_of_statuscontent_h1 = html[0..end_of_statuscontent_h1].trim();
        debug!("content_of_statuscontent_h1: {content_of_statuscontent_h1:?}.");
        if content_of_statuscontent_h1.contains("签到成功") {
            return Ok(PreSignResult::Susses);
        }
    }
    let mut captcha_id_and_location = Dioption::None;
    if let Some(location) = LocationWithRange::find_in_html(&html) {
        captcha_id_and_location.push_second(location);
    }
    if let Some(captcha_id) = find_captcha(session, &html) {
        captcha_id_and_location.push_first(captcha_id);
    }
    let response_of_analysis = protocol::analysis(session, active_id)?;
    let data = response_of_analysis.into_string().unwrap();
    let code = {
        let start_of_code = data.find("code='+'").unwrap() + 8;
        let data = &data[start_of_code..data.len()];
        let end_of_code = data.find('\'').unwrap();
        &data[0..end_of_code]
    };
    debug!("code: {code:?}");
    let _response_of_analysis2 = protocol::analysis2(session, code)?;
    debug!(
        "analysis 结果：{}",
        _response_of_analysis2.into_string().unwrap()
    );
    std::thread::sleep(std::time::Duration::from_millis(500));
    Ok(PreSignResult::Data(captcha_id_and_location))
}
pub struct PPTSignHelper {
    url: String,
}
impl PPTSignHelper {
    pub fn url(&self) -> &str {
        &self.url
    }
    pub fn get(&self, agent: &Agent) -> Result<Response, Box<ureq::Error>> {
        Ok(agent.get(self.url()).call()?)
    }
    pub fn with_enc2(mut self, enc2: &str) -> Self {
        self.url += "&enc2=";
        self.url += enc2;
        self
    }
    pub fn with_validate(mut self, validate: &str) -> Self {
        self.url += "&validate=";
        self.url += validate;
        self
    }
}
impl Deref for PPTSignHelper {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.url()
    }
}
impl DerefMut for PPTSignHelper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.url
    }
}

impl From<String> for PPTSignHelper {
    fn from(s: String) -> Self {
        Self { url: s }
    }
}

pub fn try_secondary_verification<Sign: SignTrait>(
    agent: &ureq::Agent,
    url: PPTSignHelper,
    captcha_id: &Option<CaptchaId>,
) -> Result<SignResult, cxlib_error::Error> {
    let r = url.get(agent)?;
    match Sign::guess_sign_result_by_text(&r.into_string().unwrap()) {
        SignResult::Fail { msg } => {
            if msg.starts_with("validate") {
                // 这里假设了二次验证只有在“签到成功”的情况下出现。
                let url = if msg.len() > 9 {
                    let enc2 = &msg[9..msg.len()];
                    debug!("enc2: {enc2:?}");
                    url.with_enc2(enc2)
                } else {
                    url
                };
                let captcha_id = if let Some(captcha_id) = captcha_id {
                    ProtocolItem::CaptchaId.update(captcha_id);
                    captcha_id
                } else {
                    warn!("未找到滑块 ID, 使用内建值。");
                    &ProtocolItem::CaptchaId.to_string()
                };
                let url_param = cxlib_captcha::utils::captcha_solver(agent, captcha_id)?;
                let r = {
                    let url = url.with_validate(&url_param);
                    let r = url.get(agent)?;
                    RawSign::guess_sign_result_by_text(&r.into_string().unwrap())
                };
                Ok(r)
            } else {
                Ok(SignResult::Fail { msg })
            }
        }
        susses => Ok(susses),
    }
}
