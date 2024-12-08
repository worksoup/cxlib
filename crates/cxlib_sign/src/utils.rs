use crate::{protocol, PreSignResult, SignResult, SignTrait};
use cxlib_activity::RawSign;
use cxlib_captcha::{utils::find_captcha, CaptchaId, DEFAULT_CAPTCHA_TYPE};
use cxlib_error::SignError;
use cxlib_protocol::{ProtocolItem, ProtocolItemTrait};
use cxlib_types::{Dioption, LocationWithRange};
use cxlib_user::Session;
use log::{debug, trace, warn};
use std::ops::{Deref, DerefMut};
use ureq::{Agent, Response};

pub fn analysis_after_presign(
    active_id: &str,
    session: &Session,
    response_of_presign: Response,
) -> Result<PreSignResult, SignError> {
    let html = response_of_presign
        .into_string()
        .unwrap_or_else(cxlib_error::log_panic);
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
    let captcha_id_and_location = Dioption::from((
        find_captcha(session, &html),
        LocationWithRange::find_in_html(&html),
    ));
    let response_of_analysis = protocol::analysis(session, active_id)?;
    let data = response_of_analysis
        .into_string()
        .expect("Convert response of analysis into String failed.");
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
        _response_of_analysis2
            .into_string()
            .expect("Convert response of analysis2 into String failed.")
    );
    // 防止行为检测导致失败。
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
    pub fn path_enc_by_pre_sign_result_msg(self, msg: String) -> Self {
        if msg.len() > 9 {
            let enc2 = &msg[9..msg.len()];
            debug!("enc2: {enc2:?}");
            self.with_enc2(enc2)
        } else {
            self
        }
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
pub fn secondary_verification(
    agent: &Agent,
    url: PPTSignHelper,
    captcha_id: Option<&CaptchaId>,
) -> Result<SignResult, SignError> {
    let captcha_id = if let Some(captcha_id) = captcha_id {
        captcha_id
    } else {
        warn!("未找到 CaptchaId, 使用内建值。");
        &ProtocolItem::CaptchaId.get()
    };
    let url_param = DEFAULT_CAPTCHA_TYPE.solve_captcha(agent, captcha_id, url.url())?;
    let r = {
        let url = url.with_validate(&url_param);
        let r = url.get(agent)?;
        RawSign::guess_sign_result_by_text(&r.into_string().unwrap_or_else(cxlib_error::log_panic))
    };
    Ok(r)
}
pub fn try_secondary_verification<Sign: SignTrait + ?Sized>(
    agent: &Agent,
    url: PPTSignHelper,
    captcha_id: Option<&CaptchaId>,
) -> Result<SignResult, SignError> {
    let r = url.get(agent)?;
    match Sign::guess_sign_result_by_text(&r.into_string().unwrap_or_else(cxlib_error::log_panic)) {
        SignResult::Fail { msg } => {
            if msg.starts_with("validate") {
                // 这里假设了二次验证只有在“签到成功”的情况下出现。
                let url = url.path_enc_by_pre_sign_result_msg(msg);
                secondary_verification(agent, url, captcha_id)
            } else {
                Ok(SignResult::Fail { msg })
            }
        }
        success => Ok(success),
    }
}
