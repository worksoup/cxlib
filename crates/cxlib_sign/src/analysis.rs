use cx_gizmo_types::OptionPair;
use cxlib_error_utils::CxlibResultUtils;
use cxlib_protocol::collect::{AnalysisResultResult, CaptchaProtocolTrait, SignProtocolTrait};
use cxlib_types::Session;

use crate::AsRaw;

pub trait Analysis: AsRaw {
    type AnalysisData: ?Sized;
    fn analysis<CaptchaProtocol: CaptchaProtocolTrait, SignProtocol: SignProtocolTrait>(
        &self,
        session: &Session,
        analysis_data: &Self::AnalysisData,
    ) -> Result<AnalysisResultResult, crate::SignError> {
        let _ = analysis_data;
        let captcha_id = CaptchaProtocol::get_captcha_id(session).log_ok();
        let raw_sign = self.as_inner();
        let active_id = raw_sign.active_id();
        let result_of_analysis: cxlib_protocol::collect::AnalysisResult =
            SignProtocol::analysis(session, active_id)?;
        let _response_of_analysis2 = result_of_analysis.analysis2::<SignProtocol>(session)?;
        log::debug!("analysis 结果：{_response_of_analysis2}",);
        // TODO: 这里的 referer_url 可能需要根据实际情况调整。
        let referer_url = format!(
            "https://mobilelearn.chaoxing.com/page/sign/signIn?courseId={}&classId={}&activeId={}&fid=0&timetable=0",
            raw_sign.course().id(),
            raw_sign.course().class_id(),
            raw_sign.active_id()
        );
        let pre_sign_result_data = OptionPair::from_tuple((captcha_id, None));
        Ok(AnalysisResultResult::Data {
            url: referer_url,
            data: pre_sign_result_data,
        })
    }
}
