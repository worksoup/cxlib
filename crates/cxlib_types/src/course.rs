pub use cxlib_error::CourseError;

use cxlib_error::{ActivityError, AgentError, MaybeFatalError};
use cxlib_protocol::collect::types as protocol;
use log::warn;
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::Entry, HashMap},
    fmt::Display,
    sync::{Arc, Mutex},
};
use ureq::Agent;

use crate::{Activity, LocationWithRange, OtherActivity, RawSign, Session};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Course {
    id: i64,
    class_id: i64,
    teacher: String,
    image_url: String,
    name: String,
}

impl Display for Course {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "班级号：{}, 课程号: {}, 课程名: {}, 任课教师: {}",
            self.class_id, self.id, self.name, self.teacher
        )
    }
}

impl Course {
    pub fn get_from_sessions<'a, Sessions: Iterator<Item = &'a Session>>(
        sessions: Sessions,
    ) -> Result<HashMap<Course, Vec<Session>>, CourseError> {
        let mut handles = Vec::new();
        for session in sessions {
            let session_ = session.clone();
            let handle = std::thread::spawn(move || -> Result<Vec<Course>, CourseError> {
                session_.get_courses()
            });
            handles.push((handle, session));
        }
        let mut courses = HashMap::<_, Vec<_>>::new();
        for (handle, session) in handles {
            let r = handle.join().unwrap();
            let courses_ = match r {
                Ok(c) => c,
                Err(e) => {
                    if e.is_fatal() {
                        return Err(e);
                    } else {
                        warn!(
                            "未能获取用户[{}]的课程，错误信息：{e}.",
                            session.get_stu_name()
                        );
                        Default::default()
                    }
                }
            };
            for course in courses_ {
                let entry = courses.entry(course);
                match entry {
                    Entry::Occupied(mut entry) => {
                        entry.get_mut().push(session.clone());
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(vec![session.clone()]);
                    }
                }
            }
        }
        Ok(courses)
    }
    pub fn new(id: i64, class_id: i64, teacher: &str, image_url: &str, name: &str) -> Course {
        Course {
            id,
            class_id,
            teacher: teacher.into(),
            image_url: image_url.into(),
            name: name.into(),
        }
    }
    pub fn get_id(&self) -> i64 {
        self.id
    }
    pub fn get_class_id(&self) -> i64 {
        self.class_id
    }
    pub fn get_teacher(&self) -> &str {
        &self.teacher
    }
    pub fn get_image_url(&self) -> &str {
        &self.image_url
    }
    pub fn get_name(&self) -> &str {
        &self.name
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
        let r = protocol::get_location_log(session, (self.get_id(), self.get_class_id()))?;
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
    pub fn get_activities(&self, session: &Session) -> Result<Vec<Activity>, ActivityError> {
        let r = protocol::active_list(session, (self.get_id(), self.get_class_id()))?;
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
