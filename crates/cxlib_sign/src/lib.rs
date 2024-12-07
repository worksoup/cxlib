use log::{info, warn};
use std::collections::HashMap;
use std::ops::Add;

use crate::protocol::{general_sign_url, signcode_sign_url};
use crate::utils::{try_secondary_verification, PPTSignHelper};
use cxlib_activity::RawSign;
use cxlib_captcha::CaptchaId;
use cxlib_error::{Error, UnwrapOrLogPanic};
use cxlib_protocol::{ProtocolItem, ProtocolItemTrait};
use cxlib_types::{Course, Dioption, LocationWithRange};
use cxlib_user::Session;
use serde::Deserialize;

pub mod protocol;
pub mod utils;

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
    type PreSignData: ?Sized;
    type Data: ?Sized;
    fn sign_url(
        &self,
        session: &Session,
        pre_sign_data: &Self::PreSignData,
        data: &Self::Data,
    ) -> PPTSignHelper;
    /// 获取各签到类型内部对原始签到类型的引用。
    /// [`RawSign`] 的各字段均为 `pub`,
    /// 故可以通过本函数获取一些签到通用的信息。
    fn as_inner(&self) -> &RawSign;
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
    fn get_sign_state(&self, session: &Session) -> Result<SignState, cxlib_error::Error> {
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
        } = r.into_json().unwrap_or_else(cxlib_error::log_panic);
        Ok(status.into())
    }
    /// 通过签到结果的字符串判断签到结果如何。
    fn guess_sign_result_by_text(text: &str) -> SignResult {
        match text {
            "success" => SignResult::Susses,
            msg => {
                if msg.is_empty() {
                    SignResult::Fail {
                        msg:
                        "错误信息为空，根据有限的经验，这通常意味着二维码签到的 `enc` 字段已经过期。".into()
                    }
                } else if msg == "您已签到过了" {
                    SignResult::Susses
                } else {
                    SignResult::Fail { msg: msg.into() }
                }
            }
        }
    }
    /// 预签到。
    fn pre_sign(
        &self,
        session: &Session,
        pre_sign_data: &Self::PreSignData,
    ) -> Result<PreSignResult, cxlib_error::Error> {
        let _ = pre_sign_data;
        self.as_inner().pre_sign(session, &())
    }
    fn pre_check_data(
        &self,
        session: &Session,
        data: &Self::Data,
    ) -> Result<Result<(), SignResult>, cxlib_error::Error> {
        let _ = session;
        let _ = data;
        Ok(Ok(()))
    }
    /// 本函数是否会发生未定义行为取决于 [`is_ready_for_sign`](SignTrait::is_ready_for_sign) 的实现，
    /// 调用 [`is_ready_for_sign`](SignTrait::is_ready_for_sign) 进行判断，如果真，则调用 [`sign_unchecked`](SignTrait::sign_unchecked), 否则返回
    /// [`SignResult::Fail`]{msg: "签到未准备好！".to_string()}
    fn sign(
        &self,
        session: &Session,
        pre_sign_result: PreSignResult,
        pre_sign_data: &Self::PreSignData,
        data: &Self::Data,
    ) -> Result<SignResult, cxlib_error::Error> {
        match pre_sign_result {
            PreSignResult::Susses => Ok(SignResult::Susses),
            PreSignResult::Data(ref pre_sign_result_data) => {
                match self.pre_check_data(session, data)? {
                    Ok(_) => {
                        let url = self.sign_url(session, pre_sign_data, data);
                        if let Some(captcha_id) = pre_sign_result_data.first() {
                            try_secondary_verification::<Self>(session, url, captcha_id)
                        } else {
                            warn!("未找到 CaptchaId, 使用内建值。");
                            try_secondary_verification::<Self>(
                                session,
                                url,
                                &ProtocolItem::CaptchaId.get(),
                            )
                        }
                    }
                    Err(msg) => Ok(msg),
                }
            }
        }
    }
    /// 预签到并签到。
    fn pre_sign_and_sign(
        &self,
        session: &Session,
        pre_sign_data: &Self::PreSignData,
        data: &Self::Data,
    ) -> Result<SignResult, cxlib_error::Error> {
        let r = self.pre_sign(session, pre_sign_data)?;
        self.sign(session, r, pre_sign_data, data)
    }
}

impl SignTrait for RawSign {
    type PreSignData = ();
    type Data = ();

    fn sign_url(&self, session: &Session, _: &(), _: &()) -> PPTSignHelper {
        general_sign_url(session, &self.active_id)
    }

    fn as_inner(&self) -> &RawSign {
        self
    }
    fn pre_sign(&self, session: &Session, _: &()) -> Result<PreSignResult, cxlib_error::Error> {
        let active_id = self.active_id.as_str();
        let uid = session.get_uid();
        let response_of_pre_sign =
            protocol::pre_sign(session, self.course.clone(), active_id, uid)?;
        info!("用户[{}]预签到已请求。", session.get_stu_name());
        utils::analysis_after_presign(active_id, session, response_of_pre_sign)
    }
}

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
//noinspection ALL
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
    c: Option<String>,
}
impl SignDetail {
    pub fn new(
        is_photo_value: i64,
        is_refresh_qrcode_value: i64,
        sign_code: Option<String>,
    ) -> SignDetail {
        SignDetail {
            is_photo: is_photo_value > 0,
            is_refresh_qrcode: is_refresh_qrcode_value > 0,
            c: sign_code,
        }
    }
    pub fn is_photo(&self) -> bool {
        self.is_photo
    }
    pub fn is_refresh_qrcode(&self) -> bool {
        self.is_refresh_qrcode
    }
    pub fn sign_code(&self) -> Option<&str> {
        self.c.as_deref()
    }
}
/// 针对同一个签到，但不同 Session 的处理程序。
pub trait SignnerTrait<T: SignTrait> {
    type ExtData<'e>;
    fn sign<'a, Sessions: Iterator<Item = &'a Session> + Clone>(
        &mut self,
        sign: &mut T,
        sessions: Sessions,
    ) -> Result<HashMap<&'a Session, SignResult>, Error>;
    fn sign_single(
        &mut self,
        sign: &mut T,
        session: &Session,
        extra_data: Self::ExtData<'_>,
    ) -> Result<SignResult, Error>;
}

/// 为手势签到和签到码签到实现的一个特型，方便复用代码。
///
/// 这两种签到除签到码格式以外没有任何不同之处。
pub trait GestureOrSigncodeSignTrait: Ord {
    fn as_inner(&self) -> &RawSign;
    /// 检查签到码是否正确而不进行签到。
    ///
    /// 如果是手势签到，九宫格对应数字如下：
    /// ``` matlab
    /// 1 2 3
    /// 4 5 6
    /// 7 8 9
    /// ```
    fn check_signcode(
        session: &Session,
        active_id: &str,
        signcode: &str,
    ) -> Result<Result<(), SignResult>, Error> {
        #[derive(Deserialize)]
        struct CheckR {
            #[allow(unused)]
            result: i64,
        }
        let CheckR { result } = protocol::check_signcode(session, active_id, signcode)?
            .into_json()
            .unwrap_or_log_panic();
        if result == 1 {
            Ok(Ok(()))
        } else {
            Ok(Err(SignResult::Fail {
                msg: "签到码或手势不正确".into(),
            }))
        }
    }
}
impl<T: GestureOrSigncodeSignTrait> SignTrait for T {
    type PreSignData = ();
    type Data = str;

    fn sign_url(
        &self,
        session: &Session,
        _: &Self::PreSignData,
        data: &Self::Data,
    ) -> PPTSignHelper {
        signcode_sign_url(session, &self.as_inner().active_id, data)
    }

    fn as_inner(&self) -> &RawSign {
        <Self as GestureOrSigncodeSignTrait>::as_inner(self)
    }
}
