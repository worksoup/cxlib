use crate::sign::CaptchaId;
use cxlib_sign::utils::secondary_verification;
use cxlib_sign::{PreSignResult, SignResult, SignTrait};
use cxlib_types::{Location, LocationWithRange};
use cxlib_user::Session;
use log::{error, warn};

pub fn sign_unchecked_with_location<T: SignTrait<Data = Location>>(
    sign: &T,
    pre_sign_data: &<T as SignTrait>::PreSignData,
    location: &Location,
    preset_location: &Option<LocationWithRange>,
    captcha_id: &CaptchaId,
    session: &Session,
) -> Result<SignResult, cxlib_error::Error> {
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
        let url = sign.sign_url(session, &pre_sign_data, &location);
        let r = url.get(session)?;
        match T::guess_sign_result_by_text(&r.into_string().unwrap_or_else(|e| {
            error!("{e}");
            panic!()
        })) {
            SignResult::Susses => return Ok(SignResult::Susses),
            SignResult::Fail { msg } => {
                if msg.starts_with("validate") {
                    // 这里假设了二次验证只有在“签到成功”的情况下出现。
                    let url = url.path_enc_by_pre_sign_result_msg(msg);
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
