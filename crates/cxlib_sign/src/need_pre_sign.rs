use cxlib_captcha::CaptchaSolverTrait;
use cxlib_protocol::collect::{AnalysisResultResult, CaptchaProtocolTrait, SignProtocolTrait};
use cxlib_types::{RawSign, Session};

use crate::{SignError, SignResult, SignTrait, analysis::Analysis};

pub trait NeedPreSign: SignTrait {
    /// 预签到。
    #[inline]
    fn pre_sign<CaptchaProtocol, SignProtocol, U>(
        &self,
        session: &Session<U>,
        pre_sign_data: &Self::PreSignData,
    ) -> Result<AnalysisResultResult, SignError>
    where
        CaptchaProtocol: CaptchaProtocolTrait,
        SignProtocol: SignProtocolTrait,
    {
        let _ = pre_sign_data;
        <RawSign as NeedPreSign>::pre_sign::<CaptchaProtocol, SignProtocol, _>(
            self.as_inner(),
            session,
            &(),
        )
    }
}

impl NeedPreSign for RawSign {
    fn pre_sign<CaptchaProtocol, SignProtocol, U>(
        &self,
        session: &Session<U>,
        _: &(),
    ) -> Result<AnalysisResultResult, SignError>
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
        log::info!("用户[{}]预签到已请求。", session.name());
        Ok(response_of_presign.analysis::<CaptchaProtocol, SignProtocol>(session, active_id)?)
    }
}

impl<T> Analysis for T
where
    T: NeedPreSign,
{
    type AnalysisData = T::PreSignData;
    #[inline]
    fn analysis<CaptchaProtocol: CaptchaProtocolTrait, SignProtocol: SignProtocolTrait, U>(
        &self,
        session: &Session<U>,
        pre_sign_data: &Self::AnalysisData,
    ) -> Result<AnalysisResultResult, crate::SignError> {
        self.pre_sign::<CaptchaProtocol, SignProtocol, _>(session, pre_sign_data)
    }
}
