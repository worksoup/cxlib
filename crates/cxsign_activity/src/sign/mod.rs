mod gesture;
mod location;
mod normal;
mod photo;
mod qrcode;
mod raw;
mod signcode;

pub use gesture::*;
pub use location::*;
pub use normal::*;
pub use photo::*;
pub use qrcode::*;
pub use raw::*;
pub use signcode::*;
use std::ops::Add;

use cxsign_types::{Course, Dioption, LocationWithRange};
use cxsign_user::Session;
use serde::Deserialize;

pub type CaptchaId = String;

/// # [`PreSignResult`]
/// 预签到结果，可能包含了一些签到时需要的信息。
pub enum PreSignResult {
    Susses,
    Data(Dioption<CaptchaId, LocationWithRange>),
}
impl PreSignResult {
    pub fn is_susses(&self) -> bool {
        match self {
            PreSignResult::Susses => true,
            PreSignResult::Data(_) => false,
        }
    }
    pub fn to_result(self) -> SignResult {
        match self {
            PreSignResult::Susses => SignResult::Susses,
            PreSignResult::Data(_) => unreachable!(),
        }
    }
}
/// # [`SignTrait`]
/// 所有的签到均实现了该 trait, 方便统一签到的流程。
///
/// 目前的签到类型包括[手势签到](GestureSign)、
/// [签到码签到](SigncodeSign)、[位置签到](LocationSign)、
/// [普通签到](NormalSign)、[拍照签到](PhotoSign)、
/// [二维码签到](QrCodeSign)
/// （作为枚举包含了[二维码不变签到](NormalQrCodeSign)
/// 和[二维码可变签到](RefreshQrCodeSign)）
/// 以及[原始签到类型](RawSign)。
///
/// 其中原始签到类型是还未区分签到类型的签到。
///
/// 签到类型的划分主要依据前人的工作。
///
/// 细节详见各签到的文档。
pub trait SignTrait: Ord {
    /// 获取各签到类型内部对原始签到类型的引用。
    /// [`RawSign`] 的各字段均为 `pub`,
    /// 故可以通过本函数获取一些签到通用的信息。
    fn as_inner(&self) -> &RawSign;
    /// 用来判断是否可以安全调用 [`SignTrait::sign_unchecked`].
    fn is_ready_for_sign(&self) -> bool {
        true
    }
    /// 判断签到活动是否有效（目前认定两小时内未结束的签到为有效签到）。
    fn is_valid(&self) -> bool {
        let time = std::time::Duration::from_millis(self.as_inner().start_time_mills);
        let two_hours = std::time::Duration::from_secs(7200);
        self.as_inner().status_code == 1
            && std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH.add(time))
                .unwrap()
                < two_hours
    }
    /// 获取签到后状态。参见返回类型 [`SignState`].
    fn get_sign_state(&self, session: &Session) -> Result<SignState, cxsign_error::Error> {
        let r = crate::protocol::get_attend_info(session, &self.as_inner().active_id)?;
        #[derive(Deserialize)]
        struct Status {
            status: i64,
        }
        #[derive(Deserialize)]
        struct Data {
            data: Status,
        }
        let Data {
            data: Status { status },
        } = r.into_json().unwrap();
        Ok(status.into())
    }
    /// 通过签到结果的字符串判断签到结果如何。
    fn guess_sign_result_by_text(&self, text: &str) -> SignResult {
        crate::utils::guess_sign_result_by_text(text)
    }
    /// 预签到。
    fn pre_sign(&self, session: &Session) -> Result<PreSignResult, cxsign_error::Error> {
        self.as_inner().pre_sign(session)
    }
    /// # Safety
    /// 签到类型中有一些 `Option` 枚举，而本函数会使用 `unwrap_unchecked`.
    /// 调用之前请设置相关信息（通过各签到类型的方法），保险起见可以调用 [`SignTrait::is_ready_for_sign`] 进行判断。
    unsafe fn sign_unchecked(
        &self,
        session: &Session,
        pre_sign_result: PreSignResult,
    ) -> Result<SignResult, cxsign_error::Error> {
        unsafe { self.as_inner().sign_unchecked(session, pre_sign_result) }
    }
    /// 本函数是否会发生未定义行为取决于 [`is_ready_for_sign`](SignTrait::is_ready_for_sign) 的实现，
    /// 调用 [`is_ready_for_sign`](SignTrait::is_ready_for_sign) 进行判断，如果真，则调用 [`sign_unchecked`](SignTrait::sign_unchecked), 否则返回
    /// [`SignResult::Fail`]{msg: "签到未准备好！".to_string()}
    fn sign(
        &self,
        session: &Session,
        pre_sign_result: PreSignResult,
    ) -> Result<SignResult, cxsign_error::Error> {
        if self.is_ready_for_sign() {
            unsafe { self.sign_unchecked(session, pre_sign_result) }
        } else {
            Ok(SignResult::Fail {
                msg: "签到未准备好！".to_string(),
            })
        }
    }
    /// 预签到并签到。
    fn pre_sign_and_sign(&self, session: &Session) -> Result<SignResult, cxsign_error::Error> {
        let r = self.pre_sign(session)?;
        self.sign(session, r)
    }
}
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
/// 签到的结果。为枚举类型。
/// ``` rust
/// #[derive(Debug)]
/// pub enum SignResult {
///     Susses,
///     Fail { msg: String },
/// }
///```
#[derive(Debug)]
pub enum SignResult {
    /// 签到成功。
    Susses,
    /// 签到失败以及失败原因。
    Fail { msg: String },
}
impl SignResult {
    /// 签到是否成功。
    pub fn is_susses(&self) -> bool {
        match self {
            SignResult::Susses => true,
            SignResult::Fail { .. } => false,
        }
    }
}
/// 签到后状态。
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

/// 签到以及其他活动的原始类型。不应使用。
#[derive(Debug)]
pub struct SignActivityRaw {
    pub id: String,
    pub name: String,
    pub course: Course,
    pub other_id: String,
    pub status: i32,
    pub start_time_secs: i64,
}
/// 区分签到类型时获取的一些签到的信息。
#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SignDetail {
    is_photo: bool,
    is_refresh_qrcode: bool,
    c: String,
}

/// 为手势签到和签到码签到实现的一个特型，方便复用代码。
///
/// 这两种签到除签到码格式以外没有任何不同之处。
pub trait GestureOrSigncodeSignTrait: SignTrait {
    /// 设置签到时所需的签到码或手势。
    fn set_signcode(&mut self, signcode: String);
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
