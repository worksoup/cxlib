mod gesture_or_signcode;
mod location;
mod normal;
mod photo;
mod qrcode;

pub use gesture_or_signcode::*;
pub use location::*;
pub use normal::*;
pub use photo::*;
pub use qrcode::*;

use cxlib_protocol::collect::{TypesProtocolTrait, UserProtocolTrait};
use cxlib_sign::{PreSignResult, SignError, SignTrait};
use cxlib_types::{RawSign, Session, SignDetail};
use log::{error, warn};
use std::collections::HashMap;

pub type CaptchaId = String;

/// 总体的签到类型。是一个枚举，可以通过 [`RawSign::to_sign`] 获取。
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub enum Sign<TypesProtocol> {
    /// 拍照签到
    Photo(PhotoSign<TypesProtocol>),
    /// 普通签到
    Normal(NormalSign),
    /// 二维码签到
    QrCode(QrCodeSign),
    /// 手势签到或签到码签到
    GestureOrSigncode(GestureOrSigncodeSign),
    /// 位置签到
    Location(LocationSign),
    /// 未知
    Unknown(RawSign),
}
impl<TypesProtocol> Sign<TypesProtocol> {
    pub fn detail<SignProtocol, UserProtocol>(
        &self,
        session: &Session<UserProtocol>,
    ) -> Result<SignDetail, SignError>
    where
        TypesProtocol: TypesProtocolTrait,
        UserProtocol: UserProtocolTrait,
    {
        Ok(self
            .as_raw::<SignProtocol>()
            .get_detail::<TypesProtocol, _>(session)?)
    }
    pub fn from_raw<SignProtocol, UserProtocol>(
        raw: RawSign,
        session: &Session<UserProtocol>,
    ) -> Self
    where
        TypesProtocol: TypesProtocolTrait,
        UserProtocol: UserProtocolTrait,
    {
        if let Ok(sign_detail) = raw.get_detail::<TypesProtocol, _>(session) {
            let r#else = |e| {
                error!("{}", raw.other_id());
                error!("{}", raw.course().name());
                panic!("{e}")
            };
            match raw.other_id().parse::<u8>().unwrap_or_else(r#else) {
                0 => {
                    if sign_detail.is_photo() {
                        Sign::Photo(PhotoSign::new(raw))
                    } else {
                        Sign::Normal(NormalSign { raw_sign: raw })
                    }
                }
                1 => Sign::Unknown(raw),
                2 => {
                    let mut preset_locations = raw
                        .course()
                        .get_locations::<TypesProtocol>(session)
                        .unwrap_or_else(|e| {
                            warn!("获取预设位置失败！错误信息：{e}.");
                            HashMap::new()
                        });
                    let preset_location = preset_locations.remove(raw.active_id());
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
                3 => Sign::GestureOrSigncode(GestureOrSigncodeSign::new(true, raw)),
                4 => {
                    let mut preset_locations = raw
                        .course()
                        .get_locations::<TypesProtocol>(session)
                        .unwrap_or_else(|e| {
                            warn!("获取预设位置失败！错误信息：{e}.");
                            HashMap::new()
                        });
                    let preset_location = preset_locations.remove(raw.active_id());
                    Sign::Location(LocationSign {
                        raw_sign: raw,
                        preset_location,
                    })
                }
                5 => Sign::GestureOrSigncode(GestureOrSigncodeSign::new(false, raw)),
                _ => Sign::Unknown(raw),
            }
        } else {
            Sign::Unknown(raw)
        }
    }
    pub fn as_raw<SignProtocol>(&self) -> &RawSign {
        match self {
            Sign::Photo(a) => a.as_inner(),
            Sign::Normal(a) => a.as_inner(),
            Sign::QrCode(a) => a.as_inner(),
            Sign::GestureOrSigncode(a) => a.as_inner(),
            Sign::Location(a) => a.as_inner(),
            Sign::Unknown(a) => a.as_inner(),
        }
    }
}
