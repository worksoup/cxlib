use cxlib_error::AgentError;
use log::debug;
use std::{fmt::Display, fs::File, path::Path};
use ureq::{Agent, Body, SendBody, http::Response};

pub trait TypesProtocolTrait {
    fn active_list_url() -> &'static str {
        TypesProtocol::ACTIVE_LIST
    }
    fn get_location_log_url() -> &'static str {
        TypesProtocol::GET_LOCATION_LOG
    }
    fn sign_detail_url() -> &'static str {
        TypesProtocol::SIGN_DETAIL
    }
    fn pan_chaoxing_url() -> &'static str {
        TypesProtocol::PAN_CHAOXING
    }
    fn pan_list_url() -> &'static str {
        TypesProtocol::PAN_LIST
    }
    fn pan_token_url() -> &'static str {
        TypesProtocol::PAN_TOKEN
    }
    fn pan_upload_url() -> &'static str {
        TypesProtocol::PAN_UPLOAD
    }
    /// 查询课程活动。
    fn active_list(
        client: &Agent,
        (course_id, class_id): (i64, impl Display),
    ) -> Result<Response<Body>, AgentError> {
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();
        let url = Self::active_list_url();
        let url = format!(
            "{url}?fid=0&courseId={course_id}&classId={class_id}&showNotStartedActive=0&_={time}",
        );
        debug!("{url}");
        Ok(client.get(&url).call()?)
    }

    // 获取位置信息列表
    fn get_location_log(
        session: &Agent,
        (course_id, class_id): (i64, impl Display),
    ) -> Result<Response<Body>, AgentError> {
        let url = Self::get_location_log_url();
        Ok(session
            .get(&format!(
                "{url}?DB_STRATEGY=COURSEID&STRATEGY_PARA=courseId&courseId={course_id}&classId={class_id}",
            ))
            .call()?)
    }

    // 签到信息获取
    fn sign_detail(client: &Agent, active_id: &str) -> Result<Response<Body>, AgentError> {
        let url = Self::sign_detail_url();
        let url = format!("{url}?activePrimaryId={active_id}&type=1",);
        debug!("{url}");
        Ok(client.get(&url).call()?)
    }
    // 超星网盘页
    fn pan_chaoxing(client: &Agent) -> Result<Response<Body>, AgentError> {
        let url = Self::pan_chaoxing_url();
        Ok(client.get(url).call()?)
    }

    // 网盘列表
    fn pan_list(client: &Agent, parent_id: &str, enc: &str) -> Result<Response<Body>, AgentError> {
        let url = Self::pan_list_url();
        Ok(client
            .post(&format!(
                "{url}?puid=0&shareid=0&parentId={parent_id}&page=1&size=50&enc={enc}"
            ))
            .send_empty()?)
    }

    // 获取超星云盘的 token
    fn pan_token(client: &Agent) -> Result<Response<Body>, AgentError> {
        let url = Self::pan_token_url();
        Ok(client.get(url).call()?)
    }

    // 网盘上传接口
    fn pan_upload(
        client: &Agent,
        file: &File,
        uid: &str,
        token: &str,
        file_name: &str,
    ) -> Result<Response<Body>, AgentError> {
        use crate::multipart::{Field, PreparedFields};
        let url = Self::pan_upload_url();
        let file_ext: &Path = file_name.as_ref();
        let file_ext = file_ext.extension().and_then(|s| s.to_str()).unwrap_or("");
        let mime = mime_guess::from_ext(file_ext).first_or_octet_stream();
        let mut fields = Vec::<Field>::default();
        Field::add_stream(&mut fields, "file", file, Some(file_name), Some(mime));
        Field::add_text(&mut fields, "puid", uid);
        let mut multipart = PreparedFields::from_fields(&mut fields).unwrap();
        Ok(client
            .post(&format!("{url}?_from=mobilelearn&_token={token}"))
            .header(
                "Content-Type",
                &format!("multipart/form-data; boundary={}", multipart.get_boundary()),
            )
            .send(SendBody::from_reader(&mut multipart))?)
    }
}

pub struct TypesProtocol;

impl TypesProtocol {
    /// 查询活动
    pub const ACTIVE_LIST: &'static str =
        "https://mobilelearn.chaoxing.com/v2/apis/active/student/activelist";
    /// 获取位置信息列表
    pub const GET_LOCATION_LOG: &'static str =
        "https://mobilelearn.chaoxing.com/v2/apis/sign/getLocationLog";
    /// 签到信息获取
    pub const SIGN_DETAIL: &'static str = "https://mobilelearn.chaoxing.com/newsign/signDetail";
    /// 超星网盘页
    pub const PAN_CHAOXING: &'static str = "https://pan-yz.chaoxing.com";
    /// 网盘列表
    pub const PAN_LIST: &'static str = "https://pan-yz.chaoxing.com/opt/listres";
    /// 获取超星云盘的 token
    pub const PAN_TOKEN: &'static str = "https://pan-yz.chaoxing.com/api/token/uservalid";
    /// 网盘上传接口
    pub const PAN_UPLOAD: &'static str = "https://pan-yz.chaoxing.com/upload";
}

impl TypesProtocolTrait for TypesProtocol {}
