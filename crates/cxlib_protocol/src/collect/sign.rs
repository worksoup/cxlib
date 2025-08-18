use cxlib_error::AgentError;
use cxlib_error_utils::CxlibResultUtils;
use ureq::{Agent, Body, http::Response};

pub use types::*;

mod types {
    use cx_gizmo_types::OptionPair;
    use cxlib_base_types::UnhandledGeoAddrWithRange;
    use cxlib_error::AgentError;
    use cxlib_error_utils::CxlibResultUtils;
    use log::debug;
    use ureq::Agent;

    use crate::collect::{CaptchaId, CaptchaProtocolTrait, SignProtocolTrait};

    pub struct AnalysisResult(pub(super) String);
    impl AnalysisResult {
        #[inline]
        pub fn find_analysis2_code(&self) -> &str {
            let data = &self.0;
            {
                let start_of_code = data.find("code='+'").unwrap() + 8;
                let data = &data[start_of_code..data.len()];
                let end_of_code = data.find('\'').unwrap();
                &data[0..end_of_code]
            }
        }
        #[inline]
        pub fn analysis2<SignProtocol: SignProtocolTrait>(
            &self,
            agent: &Agent,
        ) -> Result<String, AgentError> {
            let code = self.find_analysis2_code();
            debug!("code: {code:?}");
            SignProtocol::analysis2(agent, code)
        }
    }
    //noinspection ALL
    /// 签到后状态。
    ///
    /// 可以为任意值（签到发出端可以通过网络请求手动设置为任意值）。
    #[derive(Debug)]
    #[repr(i64)]
    pub enum ValidSignState {
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
    /// 签到后状态。
    ///
    /// 可以为任意值（签到发出端可以通过网络请求手动设置为任意值）。
    #[derive(Debug)]
    pub enum SignState {
        ValidSignState(ValidSignState),
        Other(i64),
    }
    impl From<i64> for SignState {
        #[inline]
        fn from(number: i64) -> Self {
            match number {
                0 | 1 | 2 | 4 | 5 | 7 | 8 | 9 | 10 | 11 | 12 => Self::ValidSignState(unsafe {
                    std::mem::transmute::<i64, ValidSignState>(number)
                }),
                _ => Self::Other(number),
            }
        }
    }
    impl From<SignState> for i64 {
        #[inline]
        fn from(enum_value: SignState) -> Self {
            match enum_value {
                SignState::ValidSignState(valid_sign_state) => valid_sign_state as Self,
                SignState::Other(value) => value,
            }
        }
    }
    /// # [`PreSignResult`]
    /// 预签到结果，可能包含了一些签到时需要的信息。
    pub enum AnalysisResultResult {
        Susses,
        Data {
            url: String,
            data: OptionPair<CaptchaId, UnhandledGeoAddrWithRange>,
        },
    }
    impl AnalysisResultResult {}
    pub struct PreSignResultRaw {
        pub(super) html: String,
        pub(super) presign_url: String,
    }
    impl PreSignResultRaw {
        #[inline]
        pub fn find_captcha_id_or_get<CaptchaProtocol: CaptchaProtocolTrait>(
            &self,
            client: &Agent,
        ) -> Option<CaptchaId> {
            if let Some(start_of_captcha_id) = self.html.find("captchaId: '") {
                let id = &self.html[start_of_captcha_id + 12..start_of_captcha_id + 12 + 32];
                debug!("captcha_id: {id}");
                Some(id.to_string())
            } else {
                CaptchaProtocol::get_captcha_id(client).log_ok()
            }
        }
        pub fn guess_if_susses(&self) -> bool {
            let html = &self.html;
            let Some(start_of_statuscontent_h1) = html.find("id=\"statuscontent\"") else {
                return false;
            };
            let html = &html[start_of_statuscontent_h1 + 19..];
            let end_of_statuscontent_h1 = html.find("</").unwrap();
            let content_of_statuscontent_h1 = html[0..end_of_statuscontent_h1].trim();
            debug!("content_of_statuscontent_h1: {content_of_statuscontent_h1:?}.");
            content_of_statuscontent_h1.contains("签到成功")
        }
        pub fn analysis<CaptchaProtocol: CaptchaProtocolTrait, SignProtocol: SignProtocolTrait>(
            self,
            agent: &Agent,
            active_id: &str,
        ) -> Result<AnalysisResultResult, AgentError> {
            // TODO
            // 需要确定重定向后的 uri 为所需。
            // let presign_url = self.0.get_uri().to_string();
            // let html = self.0.into_body().read_to_string().log_unwrap();
            log::trace!("预签到请求结果：{}", self.html);
            if self.guess_if_susses() {
                return Ok(AnalysisResultResult::Susses);
            }
            let captcha_id_and_location = OptionPair::from((
                self.find_captcha_id_or_get::<CaptchaProtocol>(agent),
                UnhandledGeoAddrWithRange::find_in_html(&self.html),
            ));
            let result_of_analysis = SignProtocol::analysis(agent, active_id)?;
            let _response_of_analysis2 = result_of_analysis.analysis2::<SignProtocol>(agent)?;
            debug!("analysis 结果：{_response_of_analysis2}",);
            // 防止行为检测导致失败。
            std::thread::sleep(std::time::Duration::from_millis(500));
            Ok(AnalysisResultResult::Data {
                url: self.presign_url,
                data: captcha_id_and_location,
            })
        }
    }
}
pub trait SignProtocolTrait {
    #[inline]
    fn analysis_url() -> &'static str {
        SignProtocol::ANALYSIS
    }
    #[inline]
    fn analysis2_url() -> &'static str {
        SignProtocol::ANALYSIS2
    }
    #[inline]
    fn check_if_validate_url() -> &'static str {
        SignProtocol::CHECK_IF_VALIDATE
    }
    #[inline]
    fn check_signcode_url() -> &'static str {
        SignProtocol::CHECK_SIGNCODE
    }
    #[inline]
    // 获取签到详情。
    fn get_attend_info_url() -> &'static str {
        SignProtocol::GET_ATTEND_INFO
    }
    #[inline]
    // 获取活动详情。
    fn get_ppt_active_info_url() -> &'static str {
        SignProtocol::GET_PPT_ACTIVE_INFO
    }
    #[inline]
    //获取带签退的签到详情
    fn get_sign_in_out_attend_url() -> &'static str {
        SignProtocol::GET_SIGN_IN_OUT_ATTEND
    }
    #[inline]
    fn ppt_sign_url() -> &'static str {
        SignProtocol::PPT_SIGN
    }
    #[inline]
    fn pre_sign_url() -> &'static str {
        SignProtocol::PRE_SIGN
    }
    /// 新签到API 2507
    #[inline]
    fn sign_in_url() -> &'static str {
        SignProtocol::SIGN_IN
    }
    // analysis
    #[inline]
    fn analysis(agent: &Agent, active_id: &str) -> Result<AnalysisResult, AgentError> {
        let url = Self::analysis_url();
        let url = format!("{url}?vs=1&DB_STRATEGY=RANDOM&aid={active_id}");

        let s = agent
            .get(&url)
            .call()?
            .into_body()
            .read_to_string()
            .expect("Convert response of analysis into String failed.");
        Ok(AnalysisResult(s))
    }

    // analysis 2
    #[inline]
    fn analysis2(client: &Agent, code: &str) -> Result<String, AgentError> {
        let url = Self::analysis2_url();
        let url = format!("{url}?DB_STRATEGY=RANDOM&code={code}");
        Ok(client
            .get(&url)
            .call()?
            .into_body()
            .read_to_string()
            .log_unwrap())
    }
    // 检查是否需要Captcha验证。
    #[inline]
    fn check_if_validate(client: &Agent, active_id: &str) -> Result<bool, AgentError> {
        #[derive(Debug, serde::Deserialize)]
        struct Res {
            result: u8,
        }
        let url = Self::check_if_validate_url();
        let r = client
            .get(&format!(
                "{url}?DB_STRATEGY=PRIMARY_KEY&STRATEGY_PARA=activeId&activeId={active_id}&&puid="
            ))
            .call()?;
        Ok(r.into_body().read_json::<Res>().log_unwrap().result != 0)
    }
    /// 签到码检查
    ///
    /// 返回 i64, 结果为 1 表示签到码正确。
    #[inline]
    fn check_signcode(client: &Agent, active_id: &str, signcode: &str) -> Result<i64, AgentError> {
        #[derive(serde::Deserialize)]
        struct CheckR {
            #[allow(unused)]
            #[serde(rename = "result")]
            result_code: i64,
        }
        let url = Self::check_signcode_url();
        let r = client
            .get(&format!("{url}?activeId={active_id}&signCode={signcode}"))
            .call()?;
        let CheckR { result_code } = r.into_body().read_json().log_unwrap();
        Ok(result_code)
    }
    // 获取签到之后的信息，例如签到时的 ip, UA, 时间等
    // 参见 "http://mobilelearn.chaoxing.com/page/sign/signIn?courseId=$&classId=$&activeId=$&fid=$"
    /// 获取签到后状态。参见返回类型 [`SignState`].
    fn get_sign_state(client: &Agent, active_id: &str) -> Result<SignState, AgentError> {
        use serde::Deserialize;
        #[derive(Deserialize)]
        struct Status {
            status: i64,
        }
        #[derive(Deserialize)]
        struct Data {
            data: Status,
        }
        //泛雅课堂多班发放通过统一链接进入的有此参数: moreClassAttendEnc
        let url = Self::get_attend_info_url();
        let r = client
            .get(&format!(
                "{url}?activeId={active_id}&type=1&moreClassAttendEnc="
            ))
            .call()?;
        let Data {
            data: Status { status },
        } = r.into_body().read_json().log_unwrap();
        Ok(status.into())
    }
    #[inline]
    fn get_ppt_active_info(client: &Agent, active_id: &str) -> Result<Response<Body>, AgentError> {
        let url = Self::get_ppt_active_info_url();
        Ok(client.get(&format!("{url}?activeId={active_id}")).call()?)
    }
    #[inline]
    fn get_sign_in_out_attend(
        client: &Agent,
        active_id: &str,
    ) -> Result<Response<Body>, AgentError> {
        //泛雅课堂多班发放通过统一链接进入的有此参数: moreClassAttendEnc
        let url = Self::get_attend_info_url();
        Ok(client
            .get(&format!(
                "{url}?activeId={active_id}&type=1&moreClassAttendEnc="
            ))
            .call()?)
    }

    // 预签到
    #[inline]
    fn pre_sign(
        client: &Agent,
        (course_id, class_id): (i64, i64),
        active_id: &str,
        uid: &str,
    ) -> Result<PreSignResultRaw, AgentError> {
        let url = Self::pre_sign_url();
        let url = format!(
            "{url}?courseId={course_id}&classId={class_id}&activePrimaryId={active_id}&general=1&sys=1&ls=1&appType=15&&tid=&uid={uid}&ut=s&isTeacherViewOpen=0"
        );
        Ok(response_to_pre_sign_result_raw(client.get(&url).call()?))
    }
    #[inline]
    fn pre_sign_for_qrcode_sign(
        client: &Agent,
        (course_id, class_id): (i64, i64),
        active_id: &str,
        uid: &str,
        c: &str,
        enc: &str,
    ) -> Result<PreSignResultRaw, AgentError> {
        let url = Self::pre_sign_url();
        let url = format!(
            "{url}?courseId={course_id}&classId={class_id}&activePrimaryId={active_id}&general=1&sys=1&ls=1&appType=15&&tid=&uid={uid}&ut=s&isTeacherViewOpen=0&rcode={}",
            format_args!(
                "&rcode={}",
                percent_encoding::utf8_percent_encode(
                    &format!("SIGNIN:aid={active_id}&source=15&Code={c}&enc={enc}"),
                    percent_encoding::NON_ALPHANUMERIC
                )
            )
        );
        Ok(response_to_pre_sign_result_raw(client.get(&url).call()?))
    }
}
#[inline]
fn response_to_pre_sign_result_raw(response_of_presign: Response<Body>) -> PreSignResultRaw {
    let presign_url = ureq::ResponseExt::get_uri(&response_of_presign).to_string();
    let html = response_of_presign
        .into_body()
        .read_to_string()
        .log_unwrap();
    PreSignResultRaw { html, presign_url }
}
pub struct SignProtocol;
impl SignProtocol {
    /// analysis
    pub const ANALYSIS: &'static str = "https://mobilelearn.chaoxing.com/pptSign/analysis";
    /// analysis 2
    pub const ANALYSIS2: &'static str = "https://mobilelearn.chaoxing.com/pptSign/analysis2";
    // 检查是否需要Captcha验证码。
    pub const CHECK_IF_VALIDATE: &'static str =
        "https://mobilelearn.chaoxing.com/widget/sign/pcStuSignController/checkIfValidate";
    /// 签到码检查
    pub const CHECK_SIGNCODE: &'static str =
        "https://mobilelearn.chaoxing.com/widget/sign/pcStuSignController/checkSignCode";
    /// 获取签到之后的信息，例如签到时的 ip, UA, 时间等
    /// 参见 "http://mobilelearn.chaoxing.com/page/sign/signIn?courseId=$&classId=$&activeId=$&fid=$"
    pub const GET_ATTEND_INFO: &'static str =
        "https://mobilelearn.chaoxing.com/v2/apis/sign/getAttendInfo";
    pub const GET_PPT_ACTIVE_INFO: &'static str =
        "https://mobilelearn.chaoxing.com/v2/apis/active/getPPTActiveInfo";
    pub const GET_SIGN_IN_OUT_ATTEND: &'static str = "https://mobilelearn.chaoxing.com/v2/apis/sign/sign-in-out/attend-info?DB_STRATEGY=PRIMARY_KEY&STRATEGY_PARA=activeId";
    /// 签到
    pub const PPT_SIGN: &'static str = "https://mobilelearn.chaoxing.com/pptSign/stuSignajax";
    /// 新签到API
    pub const SIGN_IN: &'static str = "https://mobilelearn.chaoxing.com/v2/apis/sign/signIn";
    /// 预签到
    pub const PRE_SIGN: &'static str = "https://mobilelearn.chaoxing.com/newsign/preSign";
}
impl SignProtocolTrait for SignProtocol {}
