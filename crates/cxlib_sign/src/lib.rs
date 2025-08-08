use crate::utils::try_secondary_verification;
use cx_gizmo_types::OptionPair;
use cxlib_captcha::CaptchaSolverTrait;
use cxlib_error_utils::CxlibResultUtils;
use cxlib_protocol::{
    collect::{
        CaptchaId, CaptchaProtocolTrait, PreSignResult, SignProtocolTrait, SignState,
        ValidSignState,
    },
    utils::PPTSignHelper,
};
use cxlib_types::{CourseWithInfo, RawSign, Session, UnhandledGeoAddrWithRange};
use log::info;
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
    /// 判断签到活动是否有效
    ///
    /// 目前认定两小时内未结束的签到为有效签到。
    /// 开始时间可能不存在，则认定为无效签到。课程已经结束则返回无效。
    fn is_valid(&self) -> bool {
        !self.as_inner().class_ended()
            && if let Some(time_mills) = self.as_inner().start_time_mills() {
                let time = std::time::Duration::from_millis(*time_mills);
                let two_hours = std::time::Duration::from_secs(7200);
                1 == *self.as_inner().status_code()
                    && std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH.add(time))
                        .log_unwrap()
                        < two_hours
            } else {
                false
            }
    }
    /// 获取签到后状态。参见返回类型 [`SignState`].
    fn guess_sign_result_by_state<SignProtocol, U>(
        &self,
        session: &Session<U>,
    ) -> Result<Option<SignResult>, SignError>
    where
        SignProtocol: SignProtocolTrait,
    {
        let state = SignProtocol::get_sign_state(session, self.as_inner().active_id())?;
        let result = match state {
            SignState::ValidSignState(valid_sign_state) => match valid_sign_state {
                ValidSignState::未签 => {
                    return Ok(None);
                }
                ValidSignState::签到成功 => SignResult::Success,
                ValidSignState::教师代签 => SignResult::PartialSuccess {
                    msg: format!(
                        "用户[`{}`]签到[`{}`]为教师代签。",
                        session.name(),
                        self.as_inner().name()
                    ),
                },
                state @ (ValidSignState::请假
                | ValidSignState::病假
                | ValidSignState::事假
                | ValidSignState::公假) => SignResult::PartialSuccess {
                    msg: format!(
                        "用户[`{}`]签到[`{}`]为请假状态[`{state:?}`]。",
                        session.name(),
                        self.as_inner().name()
                    ),
                },
                state @ (ValidSignState::缺勤
                | ValidSignState::迟到
                | ValidSignState::早退
                | ValidSignState::签到已过期) => SignResult::Failure {
                    msg: format!("签到失败，状态为[`{state:?}`]。",),
                },
            },
            SignState::Other(number) => SignResult::Failure {
                msg: format!("签到状态未知（`{number}`），可能是服务端 bug。"),
            },
        };
        Ok(Some(result))
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
    fn sign<CaptchaSolver: CaptchaSolverTrait, CaptchaProtocol, SignProtocol, U>(
        &self,
        session: &Session<U>,
        pre_sign_url: &str,
        pre_sign_result_data: &OptionPair<CaptchaId, UnhandledGeoAddrWithRange>,
        pre_sign_data: &Self::PreSignData,
        data: &Self::Data,
    ) -> Result<SignResult, SignError>
    where
        CaptchaProtocol: CaptchaProtocolTrait,
        SignProtocol: SignProtocolTrait,
    {
        match self.pre_check_data(session, data)? {
            Ok(_) => {
                let url = self.sign_url::<SignProtocol, U>(session, pre_sign_data, data);
                try_secondary_verification::<CaptchaSolver, CaptchaProtocol, SignProtocol, Self>(
                    session,
                    url,
                    pre_sign_result_data.first(),
                    pre_sign_url,
                )
            }
            Err(msg) => Ok(msg),
        }
    }
    /// 检查签到状态，如果需要签到，则预签到并签到。
    fn check_state_and_do_sign<
        CaptchaSolver: CaptchaSolverTrait,
        CaptchaProtocol,
        SignProtocol,
        U,
    >(
        &self,
        session: &Session<U>,
        pre_sign_data: &Self::PreSignData,
        data: &Self::Data,
    ) -> Result<SignResult, SignError>
    where
        CaptchaProtocol: CaptchaProtocolTrait,
        SignProtocol: SignProtocolTrait,
    {
        let guess_result = self
            .as_inner()
            .guess_sign_result_by_state::<SignProtocol, U>(session)?;
        if let Some(guess_result) = guess_result {
            return Ok(guess_result);
        }
        let r = self.pre_sign::<CaptchaProtocol, SignProtocol, U>(session, pre_sign_data)?;
        match r {
            PreSignResult::Susses => Ok(SignResult::Success),
            PreSignResult::Data {
                ref url,
                data: ref pre_sign_result_data,
            } => self.sign::<CaptchaSolver, CaptchaProtocol, SignProtocol, U>(
                session,
                url,
                pre_sign_result_data,
                pre_sign_data,
                data,
            ),
        }
    }
}

impl SignTrait for RawSign {
    type PreSignData = ();
    type Data = ();

    #[inline]
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
        let response_of_presign = SignProtocol::pre_sign(
            session,
            (self.course().id(), self.course().class_id()),
            active_id,
            uid,
        )?;
        info!("用户[{}]预签到已请求。", session.name());
        Ok(response_of_presign.analysis::<CaptchaProtocol, SignProtocol>(session, active_id)?)
    }
}

/// 签到的结果。为枚举类型。
/// ``` rust
/// #[derive(Debug)]
/// pub enum SignResult {
///     Susses,
///     Failure { msg: String },
/// }
///```
#[derive(Debug)]
pub enum SignResult {
    /// 签到成功。
    Success,
    PartialSuccess {
        msg: String,
    },
    /// 签到失败以及失败原因。
    Failure {
        msg: String,
    },
}
impl SignResult {
    /// 通过签到结果的字符串判断签到结果如何。
    pub fn guess_by_text(text: &str) -> SignResult {
        match text {
            "success" => SignResult::Success,
            msg => {
                if msg.is_empty() {
                    SignResult::Failure {
                        msg:
                        "错误信息为空，根据有限的经验，这通常意味着二维码签到的 `enc` 字段已经过期。".into()
                    }
                } else if msg == "您已签到过了" {
                    SignResult::Success
                } else {
                    SignResult::Failure { msg: msg.into() }
                }
            }
        }
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
pub trait SignnerTrait<T, CaptchaSolver, CaptchaProtocol, SignProtocol>
where
    T: SignTrait,
    CaptchaProtocol: CaptchaProtocolTrait,
    SignProtocol: SignProtocolTrait,
    CaptchaSolver: CaptchaSolverTrait,
{
    type ExtData<'e>;
    fn sign<'a, U: Send + 'static, Sessions: Iterator<Item = &'a Session<U>>>(
        &mut self,
        sign: &T,
        sessions: Sessions,
    ) -> Result<HashMap<&'a Session<U>, SignResult>, SignError>;
    /// 此处不使用 self, 方便多线程实现。
    fn sign_single<U>(
        sign: &T,
        session: &Session<U>,
        extra_data: Self::ExtData<'_>,
    ) -> Result<SignResult, SignError>;
}
