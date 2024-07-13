mod gesture;
mod location;
mod normal;
mod photo;
pub mod protocol;
mod qrcode;
mod signcode;
mod utils;

pub use gesture::*;
pub use location::*;
use log::{error, warn};
pub use normal::*;
pub use photo::*;
pub use qrcode::*;
pub use signcode::*;
use std::collections::HashMap;

use cxsign_activity::RawSign;
use cxsign_sign::{PreSignResult, SignDetail, SignResult, SignState, SignTrait};
use cxsign_types::{Location, LocationWithRange};
use cxsign_user::Session;
use serde::Deserialize;

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
impl SignTrait for Sign {
    fn as_inner(&self) -> &RawSign {
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
    fn is_ready_for_sign(&self) -> bool {
        match self {
            Sign::Photo(a) => a.is_ready_for_sign(),
            Sign::Normal(a) => a.is_ready_for_sign(),
            Sign::QrCode(a) => a.is_ready_for_sign(),
            Sign::Gesture(a) => a.is_ready_for_sign(),
            Sign::Location(a) => a.is_ready_for_sign(),
            Sign::Signcode(a) => a.is_ready_for_sign(),
            Sign::Unknown(a) => a.is_ready_for_sign(),
        }
    }
    fn is_valid(&self) -> bool {
        match self {
            Sign::Photo(a) => a.is_valid(),
            Sign::Normal(a) => a.is_valid(),
            Sign::QrCode(a) => a.is_valid(),
            Sign::Gesture(a) => a.is_valid(),
            Sign::Location(a) => a.is_valid(),
            Sign::Signcode(a) => a.is_valid(),
            Sign::Unknown(a) => a.is_valid(),
        }
    }

    fn get_sign_state(&self, session: &Session) -> Result<SignState, cxsign_error::Error> {
        match self {
            Sign::Photo(a) => a.get_sign_state(session),
            Sign::Normal(a) => a.get_sign_state(session),
            Sign::QrCode(a) => a.get_sign_state(session),
            Sign::Gesture(a) => a.get_sign_state(session),
            Sign::Location(a) => a.get_sign_state(session),
            Sign::Signcode(a) => a.get_sign_state(session),
            Sign::Unknown(a) => a.get_sign_state(session),
        }
    }
    fn pre_sign(&self, session: &Session) -> Result<PreSignResult, cxsign_error::Error> {
        match self {
            Sign::Photo(a) => a.pre_sign(session),
            Sign::Normal(a) => a.pre_sign(session),
            Sign::QrCode(a) => a.pre_sign(session),
            Sign::Gesture(a) => a.pre_sign(session),
            Sign::Location(a) => a.pre_sign(session),
            Sign::Signcode(a) => a.pre_sign(session),
            Sign::Unknown(a) => a.pre_sign(session),
        }
    }
    unsafe fn sign_unchecked(
        &self,
        session: &Session,
        pre_sign_result: PreSignResult,
    ) -> Result<SignResult, cxsign_error::Error> {
        unsafe {
            match self {
                Sign::Photo(a) => a.sign_unchecked(session, pre_sign_result),
                Sign::Normal(a) => a.sign_unchecked(session, pre_sign_result),
                Sign::QrCode(a) => a.sign_unchecked(session, pre_sign_result),
                Sign::Gesture(a) => a.sign_unchecked(session, pre_sign_result),
                Sign::Location(a) => a.sign_unchecked(session, pre_sign_result),
                Sign::Signcode(a) => a.sign_unchecked(session, pre_sign_result),
                Sign::Unknown(a) => a.sign_unchecked(session, pre_sign_result),
            }
        }
    }
}
impl Sign {
    pub fn get_sign_detail(
        active_id: &str,
        session: &Session,
    ) -> Result<SignDetail, cxsign_error::Error> {
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
        } = r.into_json().unwrap();
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
                        Sign::Photo(PhotoSign {
                            raw_sign: raw,
                            photo: None,
                        })
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
                    let location = if let Some(preset_location) = preset_location.as_ref() {
                        preset_location.to_shifted_location()
                    } else {
                        Location::get_none_location()
                    };
                    let raw_sign = LocationSign {
                        raw_sign,
                        location,
                        preset_location,
                    };
                    let is_refresh = sign_detail.is_refresh_qrcode();
                    Sign::QrCode(QrCodeSign {
                        is_refresh,
                        enc: None,
                        // TODO: bad `unwrap`.
                        c: sign_detail.sign_code().unwrap().to_string(),
                        raw_sign,
                    })
                }
                3 => Sign::Gesture(GestureSign {
                    raw_sign: raw,
                    gesture: None,
                }),
                4 => {
                    let mut preset_locations = LocationWithRange::from_log(session, &raw.course)
                        .unwrap_or_else(|e| {
                            warn!("获取预设位置失败！错误信息：{e}.");
                            HashMap::new()
                        });
                    let preset_location = preset_locations.remove(&raw.active_id);
                    let location = if let Some(preset_location) = preset_location.as_ref() {
                        preset_location.to_shifted_location()
                    } else {
                        Location::get_none_location()
                    };
                    Sign::Location(LocationSign {
                        raw_sign: raw,
                        location,
                        preset_location,
                    })
                }
                5 => Sign::Signcode(SigncodeSign {
                    signcode: None,
                    raw_sign: raw,
                }),
                _ => Sign::Unknown(raw),
            }
        } else {
            Sign::Unknown(raw)
        }
    }
}

/// 为手势签到和签到码签到实现的一个特型，方便复用代码。
///
/// 这两种签到除签到码格式以外没有任何不同之处。
pub trait GestureOrSigncodeSignTrait: SignTrait {
    fn sign_with_signcode(
        &self,
        session: &Session,
        signcode: &str,
    ) -> Result<SignResult, cxsign_error::Error> {
        if Self::check_signcode(session, &self.as_inner().active_id, signcode)? {
            let r = cxsign_sign::protocol::signcode_sign(
                session,
                self.as_inner().active_id.as_str(),
                signcode,
            )?;
            Ok(Self::guess_sign_result_by_text(&r.into_string().unwrap()))
        } else {
            Ok(SignResult::Fail {
                msg: "签到码或手势不正确".into(),
            })
        }
    }
    /// 设置签到时所需的签到码或手势。
    fn set_signcode(&mut self, signcode: String);
    fn check_signcode(
        session: &Session,
        active_id: &str,
        signcode: &str,
    ) -> Result<bool, cxsign_error::Error> {
        #[derive(Deserialize)]
        struct CheckR {
            #[allow(unused)]
            result: i64,
        }
        let CheckR { result } = protocol::check_signcode(session, active_id, signcode)?
            .into_json()
            .unwrap();
        Ok(result == 1)
    }
}

impl GestureOrSigncodeSignTrait for GestureSign {
    fn set_signcode(&mut self, signcode: String) {
        self.set_gesture(signcode)
    }
}

impl GestureOrSigncodeSignTrait for SigncodeSign {
    fn set_signcode(&mut self, signcode: String) {
        SigncodeSign::set_signcode(self, signcode)
    }
}
