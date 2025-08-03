use crate::error::ActivityError;
use crate::{Activity, ClassInfo, LocationWithRange, OtherActivity, RawCourse, RawSign, Session};
use bincode::{Decode, Encode};
use cxlib_error::AgentError;
use cxlib_error_utils::CxlibResultUtils;
use cxlib_protocol::collect::{TypesProtocolTrait, UserProtocolTrait};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::{Display, write},
    ops::Deref,
    str::FromStr,
    sync::{Arc, Mutex},
};
use ureq::Agent;

/// 课程，包含一个班级信息。
#[derive(
    Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Decode, Encode,
)]
pub struct CourseWithInfo {
    course: Course,
    info: CourseInfo,
}
impl CourseWithInfo {
    // #[inline]
    // pub fn none() -> Self {
    //     Self {
    //         raw: RawCourse::none(),
    //         class_info: ClassInfo::none(),
    //     }
    // }
    // #[inline]
    // pub fn take(&mut self) -> Self {
    //     mem::replace(
    //         self,
    //         Self {
    //             raw: RawCourse::none(),
    //             class_info: ClassInfo::none(),
    //         },
    //     )
    // }
    #[inline]
    pub fn new(course: Course, info: CourseInfo) -> CourseWithInfo {
        Self { course, info }
    }
    #[inline]
    pub fn from_raw(raw: RawCourse, class_info: ClassInfo) -> CourseWithInfo {
        RawCourse::into_course(raw, class_info)
    }
    #[inline]
    pub(crate) fn new_with_fields(
        id: i64,
        class_info: ClassInfo,
        teacher: String,
        image_url: Option<String>,
        name: String,
    ) -> CourseWithInfo {
        let class_id = class_info.id();
        let ended = class_info.ended();
        Self {
            course: Course { id, class_id },
            info: CourseInfo {
                ended,
                teacher,
                image_url,
                name,
            },
        }
    }

    #[inline]
    pub fn unwrap(self) -> (Course, CourseInfo) {
        (self.course, self.info)
    }
    #[inline]
    pub fn unwrap_ref(&self) -> (&Course, &CourseInfo) {
        (&self.course, &self.info)
    }
    #[inline]
    pub fn course(&self) -> &Course {
        &self.course
    }
    #[inline]
    pub fn info(&self) -> &CourseInfo {
        &self.info
    }
    #[inline]
    pub fn id(&self) -> i64 {
        self.course.id
    }
    #[inline]
    pub fn teacher(&self) -> &str {
        &self.info.teacher
    }
    #[inline]
    pub fn image_url(&self) -> Option<&str> {
        self.info.image_url.as_ref().map(AsRef::as_ref)
    }
    #[inline]
    pub fn name(&self) -> &str {
        &self.info.name
    }
    #[inline]
    pub fn class_id(&self) -> i64 {
        self.course.class_id
    }
    #[inline]
    pub fn class_ended(&self) -> bool {
        self.info.ended
    }
}

impl CourseWithInfo {
    // TODO: 该函数需要注意：API 可能已经失效。
    pub fn get_locations<TypesProtocol>(
        &self,
        session: &Agent,
    ) -> Result<HashMap<String, LocationWithRange>, AgentError>
    where
        TypesProtocol: TypesProtocolTrait,
    {
        #[derive(Debug, Clone, Deserialize)]
        struct LocationWithRangeAndActiveId {
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
            pub fn into_location_with_range(self) -> LocationWithRange {
                LocationWithRange::new(
                    self.addr,
                    self.lon.to_string(),
                    self.lat.to_string(),
                    self.range.trim().parse().unwrap_or(100),
                )
            }
        }
        #[derive(Debug, Clone, Deserialize)]
        struct Data {
            #[serde(rename = "data")]
            data: Vec<LocationWithRangeAndActiveId>,
        }
        let r = TypesProtocol::get_location_log(session, (self.id(), self.class_id()))?;
        let data: Data = r.into_body().read_json().log_unwrap();
        let mut map = HashMap::new();
        for l in data.data {
            map.insert(l.active_id.to_string(), l.into_location_with_range());
        }
        Ok(map)
    }
}

/// # ActivityRaw
///
/// 未分类的活动类型，仅用于内部反序列化。
///
/// 请参考 [`protocol::active_list`] 的响应数据。
#[derive(Deserialize, Serialize, Clone)]
struct ActivityRaw {
    #[serde(rename = "nameOne")]
    name_one: String,
    id: i64,
    #[serde(rename = "otherId")]
    other_id: Option<String>,
    status: i32,
    #[serde(rename = "startTime")]
    start_time_mills: StartTimeMills,
}
/// 对于已经结束的课程，该字段将为空字符串。
/// 也许有可能为 null. TODO: 后续需验证。
#[derive(Deserialize, Serialize, Clone)]
#[serde(untagged)]
enum StartTimeMills {
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
/// 内部类型，用于反序列化。
///
/// 请参考 [`protocol::active_list`] 的响应数据。
#[derive(Deserialize, Serialize)]
struct Data {
    #[serde(rename = "activeList")]
    active_list: Vec<ActivityRaw>,
}
/// 内部类型，用于反序列化。
///
/// 请参考 [`protocol::active_list`] 的响应数据。
#[derive(Deserialize, Serialize)]
struct GetActivityR {
    data: Option<Data>,
}

impl CourseWithInfo {
    /// 获取该课程的活动。
    pub fn get_activities<TypesProtocol: TypesProtocolTrait, UserProtocol: UserProtocolTrait>(
        &self,
        session: &Session<UserProtocol>,
    ) -> Result<Vec<Activity>, ActivityError> {
        let mut r = TypesProtocol::active_list(session, (self.id(), self.class_id()))?;
        let r: GetActivityR = {
            #[cfg(debug_assertions)]
            {
                let r = r.body_mut().read_to_string().unwrap();
                match serde_json::from_str(&r) {
                    Ok(r) => r,
                    Err(e) => {
                        log::error!("{}/{}", session.name(), self.info().name());
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
        let class_ended = self.class_ended();
        let activities = Arc::new(Mutex::new(Vec::new()));
        if let Some(data) = r.data {
            let thread_count = 1;
            let len = data.active_list.len();
            let chunk_rest = len % thread_count;
            let chunk_count = len / thread_count + if chunk_rest == 0 { 0 } else { 1 };
            for i in 0..chunk_count {
                let ars = &data.active_list[i * thread_count..if i != chunk_count - 1 {
                    (i + 1) * thread_count
                } else {
                    len
                }];
                let mut handles = Vec::new();
                for ar in ars {
                    let ar = ar.clone();
                    let c = self.clone();
                    let activities = activities.clone();
                    let handle = std::thread::spawn(move || {
                        if let Some(oid) = ar.other_id.as_ref()
                            && let Ok(other_id) = oid.parse::<i64>()
                            && { (0..=5).contains(&other_id) }
                        {
                            let active_id = ar.id.to_string();
                            let base_sign = RawSign::new(
                                active_id,
                                c.clone(),
                                ar.name_one,
                                other_id,
                                ar.status,
                                ar.start_time_mills.some(),
                                class_ended,
                            );
                            activities
                                .lock()
                                .unwrap()
                                .push(Activity::RawSign(base_sign))
                        } else {
                            activities
                                .lock()
                                .unwrap()
                                .push(Activity::Other(OtherActivity {
                                    other_id: ar.other_id,
                                    id: ar.id.to_string(),
                                    name: ar.name_one,
                                    course: c.clone(),
                                    status_code: ar.status,
                                    start_time_mills: ar.start_time_mills.some(),
                                }))
                        }
                    });
                    handles.push(handle);
                }
                for h in handles {
                    h.join().unwrap();
                }
            }
        }
        let activities = Arc::into_inner(activities).unwrap().into_inner().unwrap();
        Ok(activities)
    }
}
impl Display for CourseWithInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "班级号:{}, 课程号: {}, 课程名: {}, 任课教师: {}",
            self.class_id(),
            self.id(),
            self.name(),
            self.teacher()
        )
    }
}
#[derive(
    Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Decode, Encode,
)]
pub struct Course {
    id: i64,
    class_id: i64,
}
impl Course {
    pub const GLOBAL: Self = Self::new(-1, -1);
    pub const fn new(id: i64, class_id: i64) -> Self {
        Self { id, class_id }
    }
    #[inline]
    pub const fn global_course() -> Self {
        Self::GLOBAL
    }
    #[inline]
    pub const fn id(&self) -> i64 {
        self.id
    }
    #[inline]
    pub const fn class_id(&self) -> i64 {
        self.class_id
    }
    #[inline]
    pub const fn invalid(&self) -> bool {
        self.id < 0 || self.class_id < 0
    }
    #[inline]
    pub const fn is_global_course(&self) -> bool {
        let g = Self::GLOBAL;
        self.id == g.id && self.class_id == g.class_id
    }
}
impl FromStr for Course {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut course = s.split(',').map(|s| s.trim());
        let id = course
            .next()
            .ok_or_else(|| "id 解析出错！".to_owned())?
            .parse()
            .map_err(|e: std::num::ParseIntError| e.to_string())?;
        let class_id = course
            .next()
            .ok_or_else(|| "class_id 解析出错！".to_owned())?
            .parse()
            .map_err(|e: std::num::ParseIntError| e.to_string())?;
        Ok(Self { id, class_id })
    }
}
impl Display for Course {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write(f, format_args!("{}, {}", self.id(), self.class_id()))
    }
}

#[derive(
    Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Decode, Encode,
)]
pub struct CourseInfo {
    ended: bool,
    teacher: String,
    image_url: Option<String>,
    name: String,
}
impl CourseInfo {
    pub fn ended(&self) -> bool {
        self.ended
    }
    pub fn teacher(&self) -> &String {
        &self.teacher
    }
    pub fn image_url(&self) -> Option<&String> {
        self.image_url.as_ref()
    }
    pub fn name(&self) -> &String {
        &self.name
    }
}
impl Deref for CourseWithInfo {
    type Target = Course;

    fn deref(&self) -> &Self::Target {
        &self.course
    }
}
