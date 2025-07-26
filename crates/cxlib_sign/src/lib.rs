use crate::utils::try_secondary_verification;
use cx_gizmo_types::OptionPair;
use cxlib_captcha::{CaptchaId, CaptchaSolver};
use cxlib_error_utils::CxlibResultUtils;
use cxlib_protocol::{
    collect::{CaptchaProtocolTrait, SignProtocolTrait},
    utils::PPTSignHelper,
};
use cxlib_types::{CourseWithInfo, LocationWithRange, RawSign, Session};
use log::info;
use serde::Deserialize;
use std::{collections::HashMap, ops::Add};

mod error;
pub mod utils;

pub use error::*;

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
    fn sign_url<SignProtocol, U>(
        &self,
        session: &Session<U>,
        pre_sign_data: &Self::PreSignData,
        data: &Self::Data,
    ) -> PPTSignHelper
    where
        SignProtocol: SignProtocolTrait;
    /// 获取各签到类型内部对原始签到类型的引用。
    /// [`RawSign`] 的各字段均为 `pub`,
    /// 故可以通过本函数获取一些签到通用的信息。
    fn as_inner(&self) -> &RawSign;
    /// 判断签到活动是否有效（目前认定两小时内未结束的签到为有效签到）。
    fn is_valid(&self) -> bool {
        let time = std::time::Duration::from_millis(*self.as_inner().start_time_mills());
        let two_hours = std::time::Duration::from_secs(7200);
        1 == *self.as_inner().status_code()
            && std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH.add(time))
                .log_unwrap()
                < two_hours
    }
    /// 获取签到后状态。参见返回类型 [`SignState`].
    fn get_sign_state<SignProtocol, U>(&self, session: &Session<U>) -> Result<SignState, SignError>
    where
        SignProtocol: SignProtocolTrait,
    {
        let r = SignProtocol::get_attend_info(session, self.as_inner().active_id())?;
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
        } = r.into_body().read_json().log_unwrap();
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
    fn pre_sign<CaptchaProtocol, SignProtocol, U>(
        &self,
        session: &Session<U>,
        pre_sign_data: &Self::PreSignData,
    ) -> Result<PreSignResult, SignError>
    where
        CaptchaProtocol: CaptchaProtocolTrait,
        SignProtocol: SignProtocolTrait,
    {
        let _ = pre_sign_data;
        <RawSign as SignTrait>::pre_sign::<CaptchaProtocol, SignProtocol, _>(
            self.as_inner(),
            session,
            &(),
        )
    }
    fn pre_check_data<UserProtocol>(
        &self,
        session: &Session<UserProtocol>,
        data: &Self::Data,
    ) -> Result<Result<(), SignResult>, SignError> {
        let _ = session;
        let _ = data;
        Ok(Ok(()))
    }
    /// 本函数是否会发生未定义行为取决于 [`is_ready_for_sign`](SignTrait::is_ready_for_sign) 的实现，
    /// 调用 [`is_ready_for_sign`](SignTrait::is_ready_for_sign) 进行判断，如果真，则调用 [`sign_unchecked`](SignTrait::sign_unchecked), 否则返回
    /// [`SignResult::Fail`]{msg: "签到未准备好！".to_string()}
    fn sign<CaptchaProtocol, SignProtocol, U>(
        &self,
        session: &Session<U>,
        pre_sign_url: &str,
        pre_sign_result_data: &OptionPair<CaptchaId, LocationWithRange>,
        pre_sign_data: &Self::PreSignData,
        captcha_solver: &CaptchaSolver,
        data: &Self::Data,
    ) -> Result<SignResult, SignError>
    where
        CaptchaProtocol: CaptchaProtocolTrait,
        SignProtocol: SignProtocolTrait,
    {
        match self.pre_check_data(session, data)? {
            Ok(_) => {
                let url = self.sign_url::<SignProtocol, U>(session, pre_sign_data, data);
                try_secondary_verification::<CaptchaProtocol, SignProtocol, Self>(
                    session,
                    url,
                    pre_sign_result_data.first(),
                    captcha_solver,
                    pre_sign_url,
                )
            }
            Err(msg) => Ok(msg),
        }
    }
    /// 预签到并签到。
    fn pre_sign_and_sign<CaptchaProtocol, SignProtocol, U>(
        &self,
        session: &Session<U>,
        pre_sign_data: &Self::PreSignData,
        captcha_solver: &CaptchaSolver,
        data: &Self::Data,
    ) -> Result<SignResult, SignError>
    where
        CaptchaProtocol: CaptchaProtocolTrait,
        SignProtocol: SignProtocolTrait,
    {
        let r = self.pre_sign::<CaptchaProtocol, SignProtocol, U>(session, pre_sign_data)?;
        match r {
            PreSignResult::Susses => Ok(SignResult::Susses),
            PreSignResult::Data {
                ref url,
                data: ref pre_sign_result_data,
            } => self.sign::<CaptchaProtocol, SignProtocol, U>(
                session,
                url,
                pre_sign_result_data,
                pre_sign_data,
                captcha_solver,
                data,
            ),
        }
    }
}

impl SignTrait for RawSign {
    type PreSignData = ();
    type Data = ();

    fn sign_url<SignProtocol, UserProtocol>(
        &self,
        session: &Session<UserProtocol>,
        _: &(),
        _: &(),
    ) -> PPTSignHelper
    where
        SignProtocol: SignProtocolTrait,
    {
        SignProtocol::general_sign_url(
            (session.uid(), session.fid(), session.name()),
            self.active_id(),
        )
    }

    fn as_inner(&self) -> &RawSign {
        self
    }
    fn pre_sign<CaptchaProtocol, SignProtocol, U>(
        &self,
        session: &Session<U>,
        _: &(),
    ) -> Result<PreSignResult, SignError>
    where
        CaptchaProtocol: CaptchaProtocolTrait,
        SignProtocol: SignProtocolTrait,
    {
        let active_id = self.active_id();
        let uid = session.uid();
        let response_of_pre_sign = SignProtocol::pre_sign(
            session,
            (self.course().id(), self.course().class_id()),
            active_id,
            uid,
        )?;
        info!("用户[{}]预签到已请求。", session.name());
        utils::analysis_after_presign::<CaptchaProtocol, SignProtocol, _>(
            active_id,
            session,
            response_of_pre_sign,
        )
    }
}

/// # [`PreSignResult`]
/// 预签到结果，可能包含了一些签到时需要的信息。
pub enum PreSignResult {
    Susses,
    Data {
        url: String,
        data: OptionPair<CaptchaId, LocationWithRange>,
    },
}
impl PreSignResult {
    pub fn is_susses(&self) -> bool {
        match self {
            PreSignResult::Susses => true,
            PreSignResult::Data { .. } => false,
        }
    }
    pub fn to_result(self) -> SignResult {
        match self {
            PreSignResult::Susses => SignResult::Susses,
            PreSignResult::Data { .. } => unreachable!(),
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
///
/// 可以为任意值（签到发出端可以通过网络请求手动设置为任意值）。
#[derive()]
#[repr(i64)]
#[non_exhaustive]
pub enum SignState {
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
impl From<i64> for SignState {
    #[inline]
    fn from(number: i64) -> Self {
        unsafe { std::mem::transmute::<i64, SignState>(number) }
    }
}
impl From<SignState> for i64 {
    #[inline]
    fn from(enum_value: SignState) -> Self {
        enum_value as Self
    }
}
/// 签到以及其他活动的原始类型。不应使用。
#[derive(Debug)]
pub struct SignActivityRaw {
    pub id: String,
    pub name: String,
    pub course: CourseWithInfo,
    pub other_id: String,
    pub status: i32,
    pub start_time_secs: i64,
}
/// 针对同一个签到，但不同 Session 的处理程序。
pub trait SignnerTrait<T, CaptchaProtocol, SignProtocol>
where
    T: SignTrait,
    CaptchaProtocol: CaptchaProtocolTrait,
    SignProtocol: SignProtocolTrait,
{
    type ExtData<'e>;
    fn sign<'a, U: Send + 'static, Sessions: Iterator<Item = &'a Session<U>>>(
        &mut self,
        sign: &T,
        sessions: Sessions,
        captcha_solver: &'static CaptchaSolver,
    ) -> Result<HashMap<&'a Session<U>, SignResult>, SignError>;
    /// 此处不使用 self, 方便多线程实现。
    fn sign_single<U>(
        sign: &T,
        session: &Session<U>,
        captcha_solver: &CaptchaSolver,
        extra_data: Self::ExtData<'_>,
    ) -> Result<SignResult, SignError>;
}
