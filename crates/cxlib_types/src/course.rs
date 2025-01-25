pub use cxlib_error::CourseError;

use cxlib_error::{ActivityError, AgentError};
use cxlib_protocol::collect::types as protocol;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::Display,
    sync::{Arc, Mutex},
};
use ureq::Agent;

use crate::{
    Activity, ClassId, ClassInfo, LocationWithRange, OtherActivity, RawCourse, RawSign, Session,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Course {
    raw: RawCourse,
    class_info: ClassInfo,
}
impl Display for Course {
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
impl Course {
    #[inline]
    pub fn new(raw: RawCourse, class_info: ClassInfo) -> Course {
        Course { raw, class_info }
    }
    #[inline]
    pub fn id(&self) -> i64 {
        self.raw.id()
    }
    #[inline]
    pub fn teacher(&self) -> &str {
        self.raw.teacher()
    }
    #[inline]
    pub fn image_url(&self) -> Option<&str> {
        self.raw.image_url()
    }
    #[inline]
    pub fn name(&self) -> &str {
        self.raw.name()
    }
    #[inline]
    pub fn class_id(&self) -> ClassId {
        self.class_info.id()
    }
    #[inline]
    pub fn class_ended(&self) -> bool {
        self.class_info.ended()
    }
}

impl Course {
    pub fn get_locations(
        &self,
        session: &Agent,
    ) -> Result<HashMap<String, LocationWithRange>, AgentError> {
        #[derive(Debug, Clone, Deserialize, Serialize)]
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
        #[derive(Debug, Clone, Deserialize, Serialize)]
        struct Data {
            #[serde(rename = "data")]
            data: Vec<LocationWithRangeAndActiveId>,
        }
        let r = protocol::get_location_log(session, (self.id(), self.class_id()))?;
        let data: Data = r.into_json().unwrap();
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
    start_time_mills: u64,
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

impl Course {
    /// 获取该课程的活动。
    pub fn get_activities(&self, session: &Session) -> Result<Vec<Activity>, ActivityError> {
        let r = protocol::active_list(session, (self.id(), self.class_id()))?;
        let r: GetActivityR = r.into_json().unwrap();
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
                        if ar.other_id.as_ref().is_some_and(|oid| {
                            let other_id_i64: i64 = oid.parse().unwrap();
                            (0..=5).contains(&other_id_i64)
                        }) {
                            let other_id = unsafe { ar.other_id.unwrap_unchecked() };
                            let active_id = ar.id.to_string();
                            let base_sign = RawSign {
                                active_id,
                                name: ar.name_one,
                                course: c.clone(),
                                other_id,
                                status_code: ar.status,
                                start_time_mills: ar.start_time_mills,
                            };
                            activities
                                .lock()
                                .unwrap()
                                .push(Activity::RawSign(base_sign))
                        } else {
                            activities
                                .lock()
                                .unwrap()
                                .push(Activity::Other(OtherActivity {
                                    id: ar.id.to_string(),
                                    name: ar.name_one,
                                    course: c.clone(),
                                    status: ar.status,
                                    start_time_mills: ar.start_time_mills,
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
