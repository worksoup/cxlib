use cxlib_error::AgentError;
use cxlib_error_utils::CxlibResultUtils;
use log::debug;
use ureq::Agent;

mod types_ {
    use cxlib_base_types::{SignDetail, UnhandledGeoAddrWithRange};
    use getset2::Getset2;
    use serde::Deserialize;

    /// 对于已经结束的课程，该字段将为空字符串。
    /// 也许有可能为 null. TODO: 后续需验证。
    #[derive(Deserialize, Clone)]
    #[serde(untagged)]
    pub enum StartTimeMills {
        Some(u64),
        None(String),
    }
    impl StartTimeMills {
        #[inline]
        pub fn some(&self) -> Option<u64> {
            match self {
                Self::Some(mills) => Some(*mills),
                Self::None(_) => None,
            }
        }
    }
    /// # ActivityRaw
    ///
    /// 未分类的活动类型，仅用于内部反序列化。
    ///
    /// 请参考 [`protocol::active_list`] 的响应数据。
    #[derive(Deserialize, Clone)]
    pub struct ActivityRaw {
        #[serde(rename = "nameOne")]
        pub name_one: String,
        pub id: i64,
        #[serde(rename = "otherId")]
        pub other_id: Option<String>,
        pub status: i32,
        #[serde(rename = "startTime")]
        pub start_time_mills: StartTimeMills,
    }
    /// 内部类型，用于反序列化。
    ///
    /// 请参考 [`protocol::active_list`] 的响应数据。
    #[derive(Deserialize, Getset2)]
    #[getset2(get_ref(pub))]
    pub struct Data {
        #[serde(rename = "activeList")]
        active_list: Vec<ActivityRaw>,
    }
    /// 内部类型，用于反序列化。
    ///
    /// 请参考 [`protocol::active_list`] 的响应数据。
    #[derive(Deserialize, Getset2)]
    #[getset2(get_ref(pub))]
    pub struct ActivityListRaw {
        data: Option<Data>,
    }
    #[derive(Debug, Clone, Deserialize, Getset2)]
    #[getset2(get_ref(pub))]
    pub struct LocationWithRangeAndActiveId {
        #[serde(rename = "activeid")]
        active_id: i64,
        #[serde(rename = "address")]
        addr: String,
        #[serde(rename = "longitude")]
        lon: f64,
        #[serde(rename = "latitude")]
        lat: f64,
        #[serde(rename = "locationrange")]
        range: String,
    }
    impl LocationWithRangeAndActiveId {
        #[inline]
        pub fn to_location_with_range(&self) -> UnhandledGeoAddrWithRange {
            UnhandledGeoAddrWithRange::new(
                self.addr.to_owned(),
                self.lon.to_string(),
                self.lat.to_string(),
                self.range.trim().parse().unwrap_or(100),
            )
        }
    }
    /// 课程签到所使用过的位置列表。
    #[derive(Debug, Clone, Deserialize, Getset2)]
    #[getset2(get_ref(pub))]
    pub struct LocationLogData {
        #[serde(rename = "data")]
        data: Vec<LocationWithRangeAndActiveId>,
    }
    #[derive(Deserialize)]
    pub struct SignDetailRaw {
        #[serde(rename = "ifPhoto")]
        is_photo_sign: i64,
        #[serde(rename = "ifRefreshEwm")]
        is_refresh_qrcode: i64,
        #[serde(rename = "signCode")]
        sign_code: Option<String>,
    }
    impl From<SignDetailRaw> for SignDetail {
        #[inline]
        fn from(value: SignDetailRaw) -> Self {
            SignDetail::new(
                value.is_photo_sign,
                value.is_refresh_qrcode,
                value.sign_code,
            )
        }
    }
}
pub use types_::*;
pub trait TypesProtocolTrait {
    #[inline]
    fn active_list_url() -> &'static str {
        TypesProtocol::ACTIVE_LIST
    }
    #[inline]
    fn sign_detail_url() -> &'static str {
        TypesProtocol::SIGN_DETAIL
    }
    #[inline]
    fn get_location_log_url() -> &'static str {
        TypesProtocol::GET_LOCATION_LOG
    }
    /// 查询课程活动。
    fn active_list(
        client: &Agent,
        (course_id, class_id): (i64, impl std::fmt::Display),
    ) -> Result<ActivityListRaw, AgentError> {
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
        let r = client.get(&url).call()?;
        let r = {
            #[cfg(debug_assertions)]
            {
                let mut r = r;
                let r = r.body_mut().read_to_string().unwrap();
                match serde_json::from_str(&r) {
                    Ok(r) => r,
                    Err(e) => {
                        log::error!("{course_id}/{class_id}");
                        log::error!("{r}");
                        log::error!("{e:?}");
                        panic!()
                    }
                }
            }
            #[cfg(not(debug_assertions))]
            {
                r.into_body().read_json().log_unwrap()
            }
        };
        Ok(r)
    }

    // 签到信息获取
    #[inline]
    fn sign_detail(client: &Agent, active_id: &str) -> Result<SignDetailRaw, AgentError> {
        let url = Self::sign_detail_url();
        let url = format!("{url}?activePrimaryId={active_id}&type=1",);
        debug!("{url}");
        Ok(client
            .get(&url)
            .call()?
            .into_body()
            .read_json()
            .log_unwrap())
    }

    // 获取位置信息列表
    #[inline]
    fn get_location_log(
        session: &Agent,
        (course_id, class_id): (i64, impl std::fmt::Display),
    ) -> Result<LocationLogData, AgentError> {
        let url = Self::get_location_log_url();
        let r = session
            .get(&format!(
                "{url}?DB_STRATEGY=COURSEID&STRATEGY_PARA=courseId&courseId={course_id}&classId={class_id}",
            ))
            .call()?;
        Ok(r.into_body().read_json().log_unwrap())
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
}

impl TypesProtocolTrait for TypesProtocol {}
