use crate::{SignError, SignResult};
use cxlib_captcha::CaptchaSolverTrait;
use cxlib_error_utils::CxlibResultUtils;
use cxlib_protocol::{
    collect::{CaptchaId, CaptchaProtocolTrait},
    utils::SignUrlHelper,
};
use log::warn;
use ureq::Agent;

pub fn sign_with_verification<CaptchaSolver, CaptchaProtocol>(
    agent: &Agent,
    url: SignUrlHelper,
    captcha_id: Option<&CaptchaId>,
    referer: &str,
) -> Result<SignResult, SignError>
where
    CaptchaProtocol: CaptchaProtocolTrait,
    CaptchaSolver: CaptchaSolverTrait,
{
    let captcha_id = if let Some(captcha_id) = captcha_id {
        captcha_id
    } else {
        warn!("未找到 CaptchaId, 使用内建值。");
        cxlib_protocol::collect::CAPTCHA_ID
    };
    let url_param = CaptchaSolver::solve_captcha::<CaptchaProtocol>(agent, captcha_id, referer)?;
    let r = {
        let url = url.with_validate(&url_param);
        let r = url.get(agent)?;
        SignResult::guess_by_text(&r.into_body().read_to_string().log_unwrap())
    };
    Ok(r)
}
pub fn try_secondary_verification<CaptchaSolver, CaptchaProtocol>(
    agent: &Agent,
    url: SignUrlHelper,
    captcha_id: Option<&CaptchaId>,
    referer: &str,
) -> Result<SignResult, SignError>
where
    CaptchaProtocol: CaptchaProtocolTrait,
    CaptchaSolver: CaptchaSolverTrait,
{
    let r = url.get(agent)?;
    match SignResult::guess_by_text(&r.into_body().read_to_string().log_unwrap()) {
        SignResult::Failure { msg, .. } => {
            if msg.starts_with("validate") {
                // 这里假设了二次验证只有在“签到成功”的情况下出现。
                let url = url.patch_enc_by_pre_sign_result_msg(msg);
                sign_with_verification::<CaptchaSolver, CaptchaProtocol>(
                    agent, url, captcha_id, referer,
                )
            } else {
                Ok(SignResult::Failure {
                    msg,
                    state_enum: None,
                })
            }
        }
        success => Ok(success),
    }
}
