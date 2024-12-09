mod gesture;
mod location;
mod normal;
mod photo;
pub mod protocol;
mod qrcode;
mod signcode;

pub use gesture::*;
pub use location::*;
use log::{error, warn};
pub use normal::*;
pub use photo::*;
pub use qrcode::*;
pub use signcode::*;

use cxlib_activity::RawSign;
use cxlib_error::{SignError, UnwrapOrLogPanic};
use cxlib_sign::{PreSignResult, SignDetail, SignTrait};
use cxlib_types::LocationWithRange;
use cxlib_user::Session;
use serde::Deserialize;
use std::collections::HashMap;

pub type CaptchaId = String;

/// 总体的签到类型。是一个枚举，可以通过 [`RawSign::to_sign`] 获取。
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub enum Sign {
    /// 拍照签到
    Photo(PhotoSign),
    /// 普通签到
    Normal(NormalSign),
    /// 二维码签到
    QrCode(QrCodeSign),
    /// 手势签到
    Gesture(GestureSign),
    /// 位置签到
    Location(LocationSign),
    /// 签到码签到
    Signcode(SigncodeSign),
    /// 未知
    Unknown(RawSign),
}
impl Sign {
    pub fn get_sign_detail(active_id: &str, session: &Session) -> Result<SignDetail, SignError> {
        #[derive(Deserialize)]
        struct GetSignDetailR {
            #[serde(rename = "ifPhoto")]
            is_photo_sign: i64,
            #[serde(rename = "ifRefreshEwm")]
            is_refresh_qrcode: i64,
            #[serde(rename = "signCode")]
            sign_code: Option<String>,
        }
        let r = protocol::sign_detail(session, active_id)?;
        let GetSignDetailR {
            is_photo_sign,
            is_refresh_qrcode,
            sign_code,
        } = r.into_json().unwrap_or_log_panic();
        Ok(SignDetail::new(is_photo_sign, is_refresh_qrcode, sign_code))
    }
    pub fn from_raw(raw: RawSign, session: &Session) -> Self {
        if let Ok(sign_detail) = Sign::get_sign_detail(raw.active_id.as_str(), session) {
            let r#else = |e| {
                error!("{}", raw.other_id);
                error!("{}", raw.course.get_name());
                panic!("{e}")
            };
            match raw.other_id.parse::<u8>().unwrap_or_else(r#else) {
                0 => {
                    if sign_detail.is_photo() {
                        Sign::Photo(PhotoSign { raw_sign: raw })
                    } else {
                        Sign::Normal(NormalSign { raw_sign: raw })
                    }
                }
                1 => Sign::Unknown(raw),
                2 => {
                    let mut preset_locations = LocationWithRange::from_log(session, &raw.course)
                        .unwrap_or_else(|e| {
                            warn!("获取预设位置失败！错误信息：{e}.");
                            HashMap::new()
                        });
                    let preset_location = preset_locations.remove(&raw.active_id);
                    let raw_sign = raw;
                    let raw_sign = LocationSign {
                        raw_sign,
                        preset_location,
                    };
                    let is_refresh = sign_detail.is_refresh_qrcode();
                    Sign::QrCode(QrCodeSign {
                        is_refresh,
                        // TODO: bad `unwrap`.
                        c: sign_detail.sign_code().unwrap().to_string(),
                        raw_sign,
                    })
                }
                3 => Sign::Gesture(GestureSign { raw_sign: raw }),
                4 => {
                    let mut preset_locations = LocationWithRange::from_log(session, &raw.course)
                        .unwrap_or_else(|e| {
                            warn!("获取预设位置失败！错误信息：{e}.");
                            HashMap::new()
                        });
                    let preset_location = preset_locations.remove(&raw.active_id);
                    Sign::Location(LocationSign {
                        raw_sign: raw,
                        preset_location,
                    })
                }
                5 => Sign::Signcode(SigncodeSign { raw_sign: raw }),
                _ => Sign::Unknown(raw),
            }
        } else {
            Sign::Unknown(raw)
        }
    }
    pub fn as_raw(&self) -> &RawSign {
        match self {
            Sign::Photo(a) => a.as_inner(),
            Sign::Normal(a) => a.as_inner(),
            Sign::QrCode(a) => a.as_inner(),
            Sign::Gesture(a) => a.as_inner(),
            Sign::Location(a) => a.as_inner(),
            Sign::Signcode(a) => a.as_inner(),
            Sign::Unknown(a) => a.as_inner(),
        }
    }
}
