mod base;
mod gesture;
mod location;
mod normal;
mod photo;
mod qr_code;
mod signcode;

pub use base::*;
pub use gesture::*;
pub use location::*;
pub use normal::*;
pub use photo::*;
pub use qr_code::*;
pub use signcode::*;

use crate::course::Course;
use crate::protocol;
use crate::user::session::Session;
use serde::Deserialize;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Sign {
    // 拍照签到
    Photo(PhotoSign),
    // 普通签到
    Normal(NormalSign),
    // 二维码签到
    QrCode(QrCodeSign),
    // 手势签到
    Gesture(GestureSign),
    // 位置签到
    Location(LocationSign),
    // 签到码签到
    Signcode(SigncodeSign),
    // 未知
    Unknown(BaseSign),
}
impl SignTrait for Sign {
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

    fn get_attend_info(&self, session: &Session) -> Result<SignState, ureq::Error> {
        match self {
            Sign::Photo(a) => a.get_attend_info(session),
            Sign::Normal(a) => a.get_attend_info(session),
            Sign::QrCode(a) => a.get_attend_info(session),
            Sign::Gesture(a) => a.get_attend_info(session),
            Sign::Location(a) => a.get_attend_info(session),
            Sign::Signcode(a) => a.get_attend_info(session),
            Sign::Unknown(a) => a.get_attend_info(session),
        }
    }

    fn pre_sign(&self, session: &Session) -> Result<SignResult, ureq::Error> {
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

    fn sign(&self, session: &Session) -> Result<SignResult, ureq::Error> {
        match self {
            Sign::Photo(a) => a.sign(session),
            Sign::Normal(a) => a.sign(session),
            Sign::QrCode(a) => a.sign(session),
            Sign::Gesture(a) => a.sign(session),
            Sign::Location(a) => a.sign(session),
            Sign::Signcode(a) => a.sign(session),
            Sign::Unknown(a) => a.sign(session),
        }
    }
}
#[derive(Debug)]
pub enum SignResult {
    Sussess,
    Fail { msg: String },
}

#[derive(num_enum::FromPrimitive, num_enum::IntoPrimitive)]
#[repr(i64)]
pub enum SignState {
    #[default]
    未签 = 0,
    签到成功 = 1,
    教师代签 = 2,
    请假 = 4,
    缺勤 = 5,
    病假 = 7,
    事假 = 8,
    迟到 = 9,
    早退 = 10,
    签到已过期 = 11,
    公假 = 12,
}

#[derive(Debug)]
pub struct SignActivityRaw {
    pub id: String,
    pub name: String,
    pub course: Course,
    pub other_id: String,
    pub status: i32,
    pub start_time_secs: i64,
}

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SignDetail {
    is_photo: bool,
    is_refresh_qrcode: bool,
    c: String,
}

pub trait SignTrait: Ord {
    fn is_valid(&self) -> bool;
    fn get_attend_info(&self, session: &Session) -> Result<SignState, ureq::Error>;
    fn 通过文本判断签到结果(text: &str) -> SignResult {
        match text {
            "success" => SignResult::Sussess,
            msg => SignResult::Fail {
                msg: if msg.is_empty() {
                    "错误信息为空，根据有限的经验，这通常意味着二维码签到的 `enc` 字段已经过期。"
                } else {
                    msg
                }
                .into(),
            },
        }
    }
    fn pre_sign(&self, session: &Session) -> Result<SignResult, ureq::Error>;
    fn sign(&self, session: &Session) -> Result<SignResult, ureq::Error>;
    fn get_sign_detail(active_id: &str, session: &Session) -> Result<SignDetail, ureq::Error> {
        #[derive(Deserialize)]
        struct GetSignDetailR {
            #[serde(alias = "ifPhoto")]
            is_photo_sign: i64,
            #[serde(alias = "ifRefreshEwm")]
            is_refresh_qrcode: i64,
            #[serde(alias = "signCode")]
            sign_code: Option<String>,
        }
        let r = protocol::sign_detail(session, active_id)?;
        let GetSignDetailR {
            is_photo_sign,
            is_refresh_qrcode,
            sign_code,
        } = r.into_json().unwrap();
        Ok(SignDetail {
            is_photo: is_photo_sign > 0,
            is_refresh_qrcode: is_refresh_qrcode > 0,
            c: if let Some(c) = sign_code {
                c
            } else {
                "".into()
            },
        })
    }
}
