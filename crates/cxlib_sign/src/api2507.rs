use cx_gizmo_types::OptionPair;
use cxlib_captcha::CaptchaSolverTrait;
use cxlib_error_utils::CxlibResultUtils;
use cxlib_protocol::{
    collect::{CaptchaId, CaptchaProtocolTrait, SignProtocolTrait},
    utils::SignUrlHelper,
};
use cxlib_types::{RawSign, Session, UnhandledGeoAddrWithRange};

use crate::{SignError, SignResult, SignTrait};

pub trait SignApi2507: Ord {
    type PreSignData: ?Sized;
    type Data: ?Sized;
    fn sign_url<SignProtocol, U>(
        &self,
        session: &Session<U>,
        pre_sign_data: &Self::PreSignData,
        data: &Self::Data,
    ) -> SignUrlHelper
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
        use std::ops::Add;
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
    #[inline]
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
    fn sign<CaptchaSolver, CaptchaProtocol, SignProtocol, U>(
        &self,
        session: &Session<U>,
        pre_sign_url: &str,
        pre_sign_result_data: &OptionPair<CaptchaId, UnhandledGeoAddrWithRange>,
        pre_sign_data: &<Self as SignApi2507>::PreSignData,
        data: &<Self as SignApi2507>::Data,
    ) -> Result<SignResult, SignError>
    where
        CaptchaProtocol: CaptchaProtocolTrait,
        SignProtocol: SignProtocolTrait,
        CaptchaSolver: CaptchaSolverTrait,
    {
        match self.pre_check_data(session, data)? {
            Ok(_) => {
                let url = self.sign_url::<SignProtocol, U>(session, pre_sign_data, data);
                let need_captcha =
                    SignProtocol::check_if_validate(session, self.as_inner().active_id())?;
                if need_captcha {
                    let captcha_id = pre_sign_result_data.first();
                    crate::utils::sign_with_verification::<CaptchaSolver, CaptchaProtocol>(
                        session,
                        url,
                        captcha_id,
                        pre_sign_url,
                    )
                } else {
                    let r = url.get(session)?;
                    Ok(SignResult::guess_by_text(
                        &r.into_body().read_to_string().log_unwrap(),
                    ))
                }
            }
            Err(msg) => Ok(msg),
        }
    }
    /// 检查签到状态，如果需要签到，则预签到并签到。
    fn check_state_and_do_sign<CaptchaSolver, CaptchaProtocol, SignProtocol, U>(
        &self,
        session: &Session<U>,
        pre_sign_data: &Self::PreSignData,
        data: &Self::Data,
    ) -> Result<SignResult, SignError>
    where
        CaptchaProtocol: CaptchaProtocolTrait,
        SignProtocol: SignProtocolTrait,
        CaptchaSolver: CaptchaSolverTrait,
    {
        let raw_sign = self.as_inner();
        let state = SignProtocol::get_sign_state(session, raw_sign.active_id())?;
        let guess_result = SignResult::guess_by_state(state, session.name(), raw_sign.name());
        if let Some(guess_result) = guess_result {
            return Ok(guess_result);
        }
        let captcha_id = CaptchaProtocol::get_captcha_id(session).log_ok();
        let active_id = raw_sign.active_id();
        let result_of_analysis = SignProtocol::analysis(session, active_id)?;
        let _response_of_analysis2 = result_of_analysis.analysis2::<SignProtocol>(session)?;
        log::debug!("analysis 结果：{_response_of_analysis2}",);

        let referer_url = format!(
            "https://mobilelearn.chaoxing.com/page/sign/signIn?courseId={}&classId={}&activeId={}&fid=0&timetable=0",
            raw_sign.course().id(),
            raw_sign.course().class_id(),
            raw_sign.active_id()
        );
        // 防止行为检测导致失败。
        std::thread::sleep(std::time::Duration::from_millis(500));
        self.sign::<CaptchaSolver, CaptchaProtocol, SignProtocol, U>(
            session,
            &referer_url,
            &OptionPair::from_tuple((captcha_id, None)),
            pre_sign_data,
            data,
        )
    }
}

impl<T: SignApi2507> SignTrait for T {
    type PreSignData = <T as SignApi2507>::PreSignData;

    type Data = <T as SignApi2507>::Data;

    #[inline]
    fn is_valid(&self) -> bool {
        <Self as SignApi2507>::is_valid(self)
    }

    #[inline]
    fn pre_check_data<UserProtocol>(
        &self,
        session: &Session<UserProtocol>,
        data: &Self::Data,
    ) -> Result<Result<(), SignResult>, SignError> {
        <Self as SignApi2507>::pre_check_data(self, session, data)
    }

    #[inline]
    fn sign<CaptchaSolver, CaptchaProtocol, SignProtocol, U>(
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
        CaptchaSolver: CaptchaSolverTrait,
    {
        <Self as SignApi2507>::sign::<CaptchaSolver, CaptchaProtocol, SignProtocol, _>(
            self,
            session,
            pre_sign_url,
            pre_sign_result_data,
            pre_sign_data,
            data,
        )
    }

    #[inline]
    fn check_state_and_do_sign<CaptchaSolver, CaptchaProtocol, SignProtocol, U>(
        &self,
        session: &Session<U>,
        pre_sign_data: &Self::PreSignData,
        data: &Self::Data,
    ) -> Result<SignResult, SignError>
    where
        CaptchaProtocol: CaptchaProtocolTrait,
        SignProtocol: SignProtocolTrait,
        CaptchaSolver: CaptchaSolverTrait,
    {
        <Self as SignApi2507>::check_state_and_do_sign::<
            CaptchaSolver,
            CaptchaProtocol,
            SignProtocol,
            _,
        >(self, session, pre_sign_data, data)
    }

    #[inline]
    fn sign_url<SignProtocol, U>(
        &self,
        session: &Session<U>,
        pre_sign_data: &Self::PreSignData,
        data: &Self::Data,
    ) -> SignUrlHelper
    where
        SignProtocol: SignProtocolTrait,
    {
        <Self as SignApi2507>::sign_url::<SignProtocol, _>(self, session, pre_sign_data, data)
    }

    #[inline]
    fn as_inner(&self) -> &RawSign {
        <Self as SignApi2507>::as_inner(self)
    }
}
