use cxlib_base_types::{Geoaddr, Geolocation};
use cxlib_error::AgentError;
use ureq::{Agent, Body, http::Response};

pub trait SignProtocolTrait {
    fn analysis_url() -> &'static str {
        SignProtocol::ANALYSIS
    }
    fn analysis2_url() -> &'static str {
        SignProtocol::ANALYSIS2
    }
    fn check_if_validate_url() -> &'static str {
        SignProtocol::CHECK_IF_VALIDATE
    }
    fn check_signcode_url() -> &'static str {
        SignProtocol::CHECK_SIGNCODE
    }
    // 获取签到详情。
    fn get_attend_info_url() -> &'static str {
        SignProtocol::GET_ATTEND_INFO
    }
    // 获取活动详情。
    fn get_ppt_active_info_url() -> &'static str {
        SignProtocol::GET_PPT_ACTIVE_INFO
    }
    //获取带签退的签到详情
    fn get_sign_in_out_attend_url() -> &'static str {
        SignProtocol::GET_SIGN_IN_OUT_ATTEND
    }
    fn ppt_sign_url() -> &'static str {
        // SignProtocol::SIGN_IN
        SignProtocol::PPT_SIGN
    }
    fn pre_sign_url() -> &'static str {
        SignProtocol::PRE_SIGN
    }
    // analysis
    fn analysis(client: &Agent, active_id: &str) -> Result<Response<Body>, AgentError> {
        let url = Self::analysis_url();
        let url = format!("{url}?vs=1&DB_STRATEGY=RANDOM&aid={active_id}");
        Ok(client.get(&url).call()?)
    }

    // analysis 2
    fn analysis2(client: &Agent, code: &str) -> Result<Response<Body>, AgentError> {
        let url = Self::analysis2_url();
        let url = format!("{url}?DB_STRATEGY=RANDOM&code={code}");
        Ok(client.get(&url).call()?)
    }
    // 检查是否需要Captcha验证码。
    fn check_if_validate(client: &Agent, active_id: &str) -> Result<Response<Body>, AgentError> {
        let url = Self::check_if_validate_url();
        Ok(client
            .get(&format!(
                "{url}?DB_STRATEGY=PRIMARY_KEY&STRATEGY_PARA=activeId&activeId={active_id}&&puid="
            ))
            .call()?)
    }
    // 签到码检查
    fn check_signcode(
        client: &Agent,
        active_id: &str,
        signcode: &str,
    ) -> Result<Response<Body>, AgentError> {
        let url = Self::check_signcode_url();
        Ok(client
            .get(&format!("{url}?activeId={active_id}&signCode={signcode}"))
            .call()?)
    }
    // 获取签到之后的信息，例如签到时的 ip, UA, 时间等
    // 参见 "http://mobilelearn.chaoxing.com/page/sign/signIn?courseId=$&classId=$&activeId=$&fid=$"
    fn get_attend_info(client: &Agent, active_id: &str) -> Result<Response<Body>, AgentError> {
        //泛雅课堂多班发放通过统一链接进入的有此参数: moreClassAttendEnc
        let url = Self::get_attend_info_url();
        Ok(client
            .get(&format!(
                "{url}?activeId={active_id}&type=1&moreClassAttendEnc="
            ))
            .call()?)
    }
    fn get_ppt_active_info(client: &Agent, active_id: &str) -> Result<Response<Body>, AgentError> {
        let url = Self::get_ppt_active_info_url();
        Ok(client.get(&format!("{url}?activeId={active_id}")).call()?)
    }
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

    // 签到
    fn general_sign_url(
        (uid, fid, stu_name): (&str, &str, &str),
        active_id: &str,
    ) -> crate::utils::PPTSignHelper {
        let url = Self::ppt_sign_url();
        format!("{url}?activeId={active_id}&uid={uid}&clientip=&latitude=-1&longitude=-1&appType=15&fid={fid}&name={stu_name}").into()
    }
    fn photo_sign_url(
        (uid, fid, stu_name): (&str, &str, &str),
        active_id: &str,
        object_id: &str,
    ) -> crate::utils::PPTSignHelper {
        // NOTE 存疑。
        let url = Self::ppt_sign_url();
        format!("{url}?activeId={active_id}&uid={uid}&clientip=&useragent=&latitude=-1&longitude=-1&appType=15&fid={fid}&objectId={object_id}&name={}", percent_encoding::utf8_percent_encode(stu_name, percent_encoding::NON_ALPHANUMERIC)).into()
    }

    fn qrcode_sign_url(
        (uid, fid, stu_name): (&str, &str, &str),
        enc: &str,
        active_id: &str,
        location: Option<&Geoaddr>,
    ) -> crate::utils::PPTSignHelper {
        let url = Self::ppt_sign_url();
        // TODO: 存疑。
        if let Some(addr) = location {
            let (addr,Geolocation{lon,lat,alt}) = addr.decompose_as();
            let location_str = format!(
                r#"{{"result":"1","address":"{addr}","latitude":{lat},"longitude":{lon},"altitude":{alt}}}"#
            );
            let location_str = percent_encoding::utf8_percent_encode(
                &location_str,
                percent_encoding::NON_ALPHANUMERIC,
            )
                .to_string();
            format!(
                r#"{url}?enc={enc}&name={stu_name}&activeId={active_id}&uid={uid}&clientip=&location={location_str}&latitude=-1&longitude=-1&fid={fid}&appType=15"#
            )
        } else {
            format!(
                r#"{url}?enc={enc}&name={stu_name}&activeId={active_id}&uid={uid}&clientip=&location=&latitude=-1&longitude=-1&fid={fid}&appType=15"#
            )
        }.into()
    }
    fn location_sign_url(
        (uid, fid, stu_name): (&str, &str, &str),
        (addr, lat, lon): (&str, &str, &str),
        active_id: &str,
        is_auto_location: bool,
    ) -> crate::utils::PPTSignHelper {
        let url = Self::ppt_sign_url();
        let if_tijiao = if is_auto_location { 1 } else { 0 };
        format!("{url}?name={stu_name}&address={addr}&activeId={active_id}&uid={uid}&clientip=&latitude={lat}&longitude={lon}&fid={fid}&appType=15&ifTiJiao={if_tijiao}").into()
    }

    fn signcode_sign_url(
        (uid, fid, stu_name): (&str, &str, &str),
        active_id: &str,
        signcode: &str,
    ) -> crate::utils::PPTSignHelper {
        let url = Self::ppt_sign_url();
        format!("{url}?activeId={active_id}&uid={uid}&clientip=&latitude=-1&longitude=-1&appType=15&fid={fid}&name={stu_name}&signCode={signcode}").into()
    }

    fn general_sign(
        agent: &Agent,
        session: (&str, &str, &str),
        active_id: &str,
    ) -> Result<Response<Body>, AgentError> {
        Self::general_sign_url(session, active_id).get(agent)
    }

    fn photo_sign(
        agent: &Agent,
        session: (&str, &str, &str),
        active_id: &str,
        object_id: &str,
    ) -> Result<Response<Body>, AgentError> {
        Self::photo_sign_url(session, active_id, object_id).get(agent)
    }
    fn qrcode_sign(
        agent: &Agent,
        session: (&str, &str, &str),
        enc: &str,
        active_id: &str,
        location: Option<&Geoaddr>,
    ) -> Result<Response<Body>, AgentError> {
        Self::qrcode_sign_url(session, enc, active_id, location).get(agent)
    }
    fn location_sign(
        agent: &Agent,
        session: (&str, &str, &str),
        location: (&str, &str, &str),
        active_id: &str,
        is_auto_location: bool,
    ) -> Result<Response<Body>, AgentError> {
        Self::location_sign_url(session, location, active_id, is_auto_location).get(agent)
    }
    fn signcode_sign(
        agent: &Agent,
        session: (&str, &str, &str),
        active_id: &str,
        signcode: &str,
    ) -> Result<Response<Body>, AgentError> {
        Self::signcode_sign_url(session, active_id, signcode).get(agent)
    }
    // 预签到
    fn pre_sign(
        client: &Agent,
        (course_id, class_id): (i64, i64),
        active_id: &str,
        uid: &str,
    ) -> Result<Response<Body>, AgentError> {
        let url = Self::pre_sign_url();
        let url = format!(
            "{url}?courseId={course_id}&classId={class_id}&activePrimaryId={active_id}&general=1&sys=1&ls=1&appType=15&&tid=&uid={uid}&ut=s&isTeacherViewOpen=0"
        );
        Ok(client.get(&url).call()?)
    }
    fn pre_sign_for_qrcode_sign(
        client: &Agent,
        (course_id, class_id): (i64, i64),
        active_id: &str,
        uid: &str,
        c: &str,
        enc: &str,
    ) -> Result<Response<Body>, AgentError> {
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
        Ok(client.get(&url).call()?)
    }
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
