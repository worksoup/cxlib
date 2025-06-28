use crate::{PreSignResult, SignError, SignResult, SignTrait};
use cx_gizmo_types::OptionPair;
use cxlib_captcha::{CaptchaError, CaptchaId, utils::find_captcha};
use cxlib_error_utils::CxlibResultUtils;
use cxlib_protocol::{
    collect::{CaptchaProtocolTrait, SignProtocolTrait},
    utils::PPTSignHelper,
};
use cxlib_types::{LocationWithRange, RawSign, Session};
use log::{debug, trace, warn};
use ureq::{Agent, Body, ResponseExt, http::Response};

pub fn analysis_after_presign<
    CaptchaProtocol: CaptchaProtocolTrait,
    SignProtocol: SignProtocolTrait,
    U,
>(
    active_id: &str,
    session: &Session<U>,
    response_of_presign: Response<Body>,
) -> Result<PreSignResult, SignError> {
    // TODO
    // 需要确定重定向后的 uri 为所需。
    let presign_url = response_of_presign.get_uri().to_string();
    let html = response_of_presign
        .into_body()
        .read_to_string()
        .log_unwrap();
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
    let captcha_id_and_location = OptionPair::from((
        find_captcha::<CaptchaProtocol>(session, &html),
        LocationWithRange::find_in_html(&html),
    ));
    let response_of_analysis = SignProtocol::analysis(session, active_id)?;
    let data = response_of_analysis
        .into_body()
        .read_to_string()
        .expect("Convert response of analysis into String failed.");
    let code = {
        let start_of_code = data.find("code='+'").unwrap() + 8;
        let data = &data[start_of_code..data.len()];
        let end_of_code = data.find('\'').unwrap();
        &data[0..end_of_code]
    };
    debug!("code: {code:?}");
    let _response_of_analysis2 = SignProtocol::analysis2(session, code)?;
    debug!(
        "analysis 结果：{}",
        _response_of_analysis2
            .into_body()
            .read_to_string()
            .expect("Convert response of analysis2 into String failed.")
    );
    // 防止行为检测导致失败。
    std::thread::sleep(std::time::Duration::from_millis(500));
    Ok(PreSignResult::Data {
        url: presign_url,
        data: captcha_id_and_location,
    })
}
pub type CaptchaSolver = fn(&Agent, &str, &str) -> Result<String, CaptchaError>;
pub fn secondary_verification<CaptchaProtocol, S>(
    agent: &Agent,
    url: PPTSignHelper,
    captcha_id: Option<&CaptchaId>,
    captcha_solver: &CaptchaSolver,
    referer: &str,
) -> Result<SignResult, SignError>
where
    CaptchaProtocol: CaptchaProtocolTrait,
{
    let captcha_id = if let Some(captcha_id) = captcha_id {
        captcha_id
    } else {
        warn!("未找到 CaptchaId, 使用内建值。");
        cxlib_protocol::collect::CAPTCHA_ID
    };
    let url_param = captcha_solver(agent, captcha_id, referer)?;
    let r = {
        let url = url.with_validate(&url_param);
        let r = url.get(agent)?;
        RawSign::<S>::guess_sign_result_by_text(&r.into_body().read_to_string().log_unwrap())
    };
    Ok(r)
}
pub fn try_secondary_verification<CaptchaProtocol, S, Sign>(
    agent: &Agent,
    url: PPTSignHelper,
    captcha_id: Option<&CaptchaId>,
    captcha_solver: &CaptchaSolver,
    referer: &str,
) -> Result<SignResult, SignError>
where
    CaptchaProtocol: CaptchaProtocolTrait,
    Sign: SignTrait<S> + ?Sized,
{
    let r = url.get(agent)?;
    match Sign::guess_sign_result_by_text(&r.into_body().read_to_string().log_unwrap()) {
        SignResult::Fail { msg } => {
            if msg.starts_with("validate") {
                // 这里假设了二次验证只有在“签到成功”的情况下出现。
                let url = url.patch_enc_by_pre_sign_result_msg(msg);
                secondary_verification::<CaptchaProtocol, S>(
                    agent,
                    url,
                    captcha_id,
                    captcha_solver,
                    referer,
                )
            } else {
                Ok(SignResult::Fail { msg })
            }
        }
        success => Ok(success),
    }
}
