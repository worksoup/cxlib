mod raw;

pub use raw::*;

use cxlib_error::{ActivityError, MaybeFatalError};
use cxlib_protocol::collect::activity as protocol;
use cxlib_types::Course;
use cxlib_user::Session;
use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

fn time_delta_from_mills(mills: u64) -> chrono::TimeDelta {
    let start_time = std::time::UNIX_EPOCH + Duration::from_millis(mills);
    let now = SystemTime::now();
    let duration = now.duration_since(start_time).unwrap();
    chrono::TimeDelta::from_std(duration).unwrap()
}
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
    /// 默认排除逻辑，即 160 天内没有任何签到即排除。
    fn if_should_exclude<'a, I: IntoIterator<Item = &'a Activity>>(&self, activities: I) -> bool {
        for activity in activities {
            if let Activity::RawSign(sign) = activity {
                if time_delta_from_mills(sign.start_time_mills).num_days() < 160 {
                    return true;
                }
            }
        }
        false
    }
    /// 课程是否被排除，参数为课程 ID.
    fn is_excluded(&self, id: i64) -> bool;
    /// 获取所有被排除的课程的 ID
    fn get_excludes(&self) -> HashSet<i64>;
    /// 排除某课程，参数为课程 ID.
    fn exclude(&self, id: i64);
    /// 取消对某课程的排除，参数为课程 ID.
    fn cancel_exclude(&self, id: i64);
    /// 更新排除列表，参数为课程 ID 的列表。
    /// 在默认实现中，该函数会完全删除旧数据，并更新为新数据。
    fn update_excludes<'a, I: IntoIterator<Item = &'a i64>>(&self, excludes: I);
}
impl CourseExcludeInfoTrait for Mutex<HashSet<i64>> {
    fn is_excluded(&self, id: i64) -> bool {
        self.lock().unwrap().contains(&id)
    }

    fn get_excludes(&self) -> HashSet<i64> {
        self.lock().unwrap().clone()
    }

    fn exclude(&self, id: i64) {
        self.lock().unwrap().insert(id);
    }

    fn cancel_exclude(&self, id: i64) {
        self.lock().unwrap().remove(&id);
    }

    fn update_excludes<'a, I: IntoIterator<Item = &'a i64>>(&self, excludes: I) {
        self.lock().unwrap().clear();
        self.lock().unwrap().extend(excludes);
    }
}
impl Activity {
    /// 获取指定的**单个**课程的活动，并决定是否将该课程加入到排除列表中。
    ///
    /// 具体逻辑参见 [`CourseExcludeInfoTrait::if_should_exclude`].
    pub fn get_course_activities(
        table: &impl CourseExcludeInfoTrait,
        session: &Session,
        course: &Course,
        set_excludes: bool,
    ) -> Result<Vec<Activity>, ActivityError> {
        let activities = Self::get_list_from_course(session, course)?;
        if set_excludes {
            let id = course.get_id();
            let dont_exclude = table.if_should_exclude(&activities);
            let excluded = table.is_excluded(id);
            if dont_exclude && excluded {
                table.cancel_exclude(id);
            } else if !dont_exclude && !excluded {
                table.exclude(id);
            }
        }
        Ok(activities)
    }
    /// 获取指定课程集合的活动，并决定是否将这些课程加入到排除列表中。
    ///
    /// 当 `set_excludes` 为 `true` 时，该函数会获取所有这些课程的活动，并根据结果改变排除列表。
    ///
    /// 具体逻辑参见 [`CourseExcludeInfoTrait::if_should_exclude`].
    ///
    /// 反之，则会根据排除列表排除部分课程，以此提高获取速度。
    ///
    /// 另见：[`CourseExcludeInfoTrait`].
    fn get_activities(
        table: &impl CourseExcludeInfoTrait,
        set_excludes: bool,
        courses: HashMap<Course, Vec<Session>>,
    ) -> Result<HashMap<Activity, Vec<Session>>, ActivityError> {
        let excludes = table.get_excludes();
        let set_excludes = set_excludes || excludes.is_empty();
        let mut course_sessions_map = courses;
        let courses = course_sessions_map
            .keys()
            .filter(|course| set_excludes || !excludes.contains(&course.get_id()))
            .cloned()
            .collect::<Vec<_>>();
        let excludes = Arc::new(Mutex::new(excludes));
        let mut valid_signs = HashMap::new();
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
                    let excludes = excludes.clone();
                    let handle = std::thread::spawn(
                        move || -> Result<(Vec<Activity>, Course), ActivityError> {
                            let activities = Self::get_course_activities(
                                &*excludes,
                                &session,
                                &course,
                                set_excludes,
                            )?;
                            Ok((activities, course))
                        },
                    );
                    handles.push(handle);
                }
            }
            for h in handles {
                debug!("handle: {h:?} join.");
                let activities = h.join().unwrap();
                let activities = match activities {
                    Ok(activities) => (
                        activities.0,
                        course_sessions_map
                            .remove(&activities.1)
                            .unwrap_or_default(),
                    ),
                    Err(e) => {
                        if e.is_fatal() {
                            return Err(e);
                        } else {
                            (Default::default(), Default::default())
                        }
                    }
                };
                let mut iter = activities.0.into_iter();
                if let Some(activity) = iter.next() {
                    for activity in iter {
                        valid_signs.insert(activity, activities.1.clone());
                    }
                    valid_signs.insert(activity, activities.1);
                }
            }
        }
        if set_excludes {
            table.update_excludes(&Arc::into_inner(excludes).unwrap().into_inner().unwrap());
        }
        Ok(valid_signs)
    }
    /// 获取所有的活动。
    ///
    /// 当 `set_excludes` 为 `true` 时，该函数会获取所有活动，并根据结果改变排除列表。
    ///
    /// 具体逻辑参见 [`CourseExcludeInfoTrait::if_should_exclude`].
    ///
    /// 反之，则会根据排除列表排除部分课程，以此提高获取速度。
    ///
    /// 另见：[`CourseExcludeInfoTrait`].
    pub fn get_all_activities<'a, Sessions: Iterator<Item = &'a Session> + Clone>(
        table: &impl CourseExcludeInfoTrait,
        sessions: Sessions,
        set_excludes: bool,
    ) -> Result<HashMap<Activity, Vec<Session>>, ActivityError> {
        let courses = Course::get_courses(sessions)?;
        Self::get_activities(table, set_excludes, courses)
    }
    pub fn get_list_from_course(session: &Session, c: &Course) -> Result<Vec<Self>, ActivityError> {
        let r = protocol::active_list(session, (c.get_id(), c.get_class_id()))?;
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
