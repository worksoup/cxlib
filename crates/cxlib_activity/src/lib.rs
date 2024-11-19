#![feature(let_chains)]
#![feature(map_try_insert)]
#![allow(incomplete_features)]
#![feature(inherent_associated_types)]

pub mod protocol;
mod raw;

pub use raw::*;

use cxlib_types::Course;
use cxlib_user::Session;
use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// # Activity
///
/// 活动类型，是一个枚举，可能是一个[暂未被分类的课程签到](RawSign)，也可能是[其他活动](OtherActivity)，如通知、作业等。
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub enum Activity {
    RawSign(RawSign),
    Other(OtherActivity),
}
/// # CourseExcludeInfoTrait
/// 课程排除列表特型。在获取[活动](Activity)列表时排除部分课程的活动，以此提高加载速度。
pub trait CourseExcludeInfoTrait {
    /// 课程是否被排除，参数为课程 ID.
    fn is_excluded(&self, id: i64) -> bool;
    /// 获取所有被排除的课程的 ID
    fn get_excludes(&self) -> Vec<i64>;
    /// 排除某课程，参数为课程 ID.
    fn exclude(&self, id: i64);
    /// 取消对某课程的排除，参数为课程 ID.
    fn disable_exclude(&self, id: i64);
    /// 更新排除列表，参数为课程 ID 的列表。
    /// 在默认实现中，该函数会完全删除旧数据，并更新为新数据。
    fn update_excludes(&self, excludes: &[i64]);
}
impl Activity {
    /// 获取指定的**单个**课程的活动，并决定是否将该课程加入到排除列表中。
    ///
    /// 具体逻辑为：若该课程在 160 天内无任何活动，则排除该课程，否则取消排除。
    ///
    /// 另见：[`CourseExcludeInfoTrait`].
    pub fn get_course_activities(
        table: &impl CourseExcludeInfoTrait,
        session: &Session,
        course: &Course,
    ) -> Result<Vec<Activity>, Box<ureq::Error>> {
        let activities = Self::get_list_from_course(session, course).unwrap_or_default();
        let mut dont_exclude = false;
        for activity in &activities {
            if let Self::RawSign(sign) = activity {
                if cxlib_utils::time_delta_since_to_now(sign.start_time_mills).num_days() < 160 {
                    dont_exclude = true;
                }
            }
        }
        let id = course.get_id();
        let excluded = table.is_excluded(id);
        if dont_exclude && excluded {
            table.disable_exclude(id);
        } else if !dont_exclude && !excluded {
            table.exclude(id);
        }
        Ok(activities)
    }
    /// 获取指定课程集合的活动，并决定是否将这些课程加入到排除列表中。
    ///
    /// 当 `set_excludes` 为 `true` 时，该函数会获取所有这些课程的活动，并根据结果改变排除列表。
    ///
    /// 具体逻辑参见 [`Activity::get_course_activities`].
    ///
    /// 反之，则会根据排除列表排除部分课程，以此提高获取速度。
    ///
    /// 另见：[`CourseExcludeInfoTrait`].
    fn get_activities(
        table: &impl CourseExcludeInfoTrait,
        set_excludes: bool,
        courses: HashMap<Course, Vec<Session>>,
    ) -> Result<HashMap<Activity, Vec<Session>>, cxlib_error::Error> {
        let excludes = table.get_excludes();
        let set_excludes = set_excludes || excludes.is_empty();
        let course_sessions_map = courses;
        let courses = course_sessions_map
            .keys()
            .filter(|course| set_excludes || !excludes.contains(&course.get_id()))
            .cloned()
            .collect::<Vec<_>>();
        let excludes = Arc::new(Mutex::new(Vec::new()));
        let valid_signs = Arc::new(Mutex::new(HashMap::new()));
        let thread_count = 256;
        let len = courses.len();
        let chunk_rest = len % thread_count;
        let chunk_count = len / thread_count + if chunk_rest == 0 { 0 } else { 1 };
        for i in 0..chunk_count {
            let courses = &courses[i * thread_count..if i != chunk_count - 1 {
                (i + 1) * thread_count
            } else {
                len
            }];
            let mut handles = Vec::new();
            for course in courses {
                debug!("加载课程{course}的签到。");
                if let Some(session) = &course_sessions_map[course].first() {
                    let course = course.clone();
                    let session = (*session).clone();
                    let activities_ = Arc::clone(&valid_signs);
                    let excludes = excludes.clone();
                    let sessions = course_sessions_map[&course].clone();
                    let handle = std::thread::spawn(move || {
                        let activities =
                            Self::get_list_from_course(&session, &course).unwrap_or(vec![]);
                        // NOTE: 此处也会将没有过签到的课程排除掉。
                        // TODO: 需要修改。
                        let mut dont_exclude = false;
                        for activity in &activities {
                            if let Self::RawSign(sign) = activity {
                                if set_excludes
                                    && cxlib_utils::time_delta_since_to_now(sign.start_time_mills)
                                        .num_days()
                                        < 160
                                {
                                    dont_exclude = true;
                                }
                            }
                        }
                        for v in activities {
                            activities_.lock().unwrap().insert(v, sessions.clone());
                        }
                        debug!("course: list_activities, ok.");
                        if set_excludes && !dont_exclude {
                            excludes.lock().unwrap().push(course.get_id())
                        }
                    });
                    handles.push(handle);
                }
            }
            for h in handles {
                debug!("handle: {h:?} join.");
                h.join().unwrap();
            }
        }
        let valid_signs = Arc::into_inner(valid_signs).unwrap().into_inner().unwrap();
        if set_excludes {
            table.update_excludes(&Arc::into_inner(excludes).unwrap().into_inner().unwrap());
        }
        Ok(valid_signs)
    }
    /// 获取所有的活动。
    ///
    /// 当 `set_excludes` 为 `true` 时，该函数会获取所有活动，并根据结果改变排除列表。
    ///
    /// 具体逻辑参见 [`Activity::get_course_activities`].
    ///
    /// 反之，则会根据排除列表排除部分课程，以此提高获取速度。
    ///
    /// 另见：[`CourseExcludeInfoTrait`].
    pub fn get_all_activities<'a, Sessions: Iterator<Item = &'a Session> + Clone>(
        table: &impl CourseExcludeInfoTrait,
        sessions: Sessions,
        set_excludes: bool,
    ) -> Result<HashMap<Activity, Vec<Session>>, cxlib_error::Error> {
        let courses = Course::get_courses(sessions)?;
        Self::get_activities(table, set_excludes, courses)
    }
    pub fn get_list_from_course(
        session: &Session,
        c: &Course,
    ) -> Result<Vec<Self>, Box<ureq::Error>> {
        let r = crate::protocol::active_list(session, c.clone())?;
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
                    let c = c.clone();
                    let activities = activities.clone();
                    let handle = std::thread::spawn(move || {
                        if let Some(other_id) = ar.other_id
                            && {
                                let other_id_i64: i64 = other_id.parse().unwrap();
                                (0..=5).contains(&other_id_i64)
                            }
                        {
                            let active_id = ar.id.to_string();
                            let base_sign = RawSign {
                                active_id,
                                name: ar.name_one,
                                course: c.clone(),
                                other_id,
                                status_code: ar.status,
                                start_time_mills: ar.start_time_mills,
                            };
                            activities.lock().unwrap().push(Self::RawSign(base_sign))
                        } else {
                            activities.lock().unwrap().push(Self::Other(OtherActivity {
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

/// # OtherActivity
///
/// 除课程签到外的其他活动，如通知、作业等。
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct OtherActivity {
    pub id: String,
    pub name: String,
    pub course: Course,
    pub status: i32,
    pub start_time_mills: u64,
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
