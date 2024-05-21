use crate::{protocol, PreSignResult};
use cxsign_types::{Dioption, LocationWithRange};
use cxsign_user::Session;
use log::{debug, trace};

pub fn analysis_after_presign(
    active_id: &str,
    session: &Session,
    response_of_presign: ureq::Response,
) -> Result<PreSignResult, cxsign_error::Error> {
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
    if let Some(start_of_captcha_id) = html.find("captchaId: '") {
        let id = &html[start_of_captcha_id + 12..start_of_captcha_id + 12 + 32];
        debug!("captcha_id: {id}");
        captcha_id_and_location.push_first(id.to_string());
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
