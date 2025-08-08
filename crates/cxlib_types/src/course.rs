use crate::error::ActivityError;
use crate::{Activity, ClassInfo, UnhandledGeoAddrWithRange, OtherActivity, RawCourse, RawSign, Session};
use bincode::{Decode, Encode};
use cxlib_error::AgentError;
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
    #[inline]
    pub fn get_locations<TypesProtocol>(
        &self,
        session: &Agent,
    ) -> Result<HashMap<String, UnhandledGeoAddrWithRange>, AgentError>
    where
        TypesProtocol: TypesProtocolTrait,
    {
        let r = TypesProtocol::get_location_log(session, (self.id(), self.class_id()))?;
        let mut map = HashMap::new();
        for l in r.data() {
            map.insert(l.active_id().to_string(), l.to_location_with_range());
        }
        Ok(map)
    }
}
impl CourseWithInfo {
    /// 获取该课程的活动。
    pub fn get_activities<TypesProtocol: TypesProtocolTrait, UserProtocol: UserProtocolTrait>(
        &self,
        session: &Session<UserProtocol>,
    ) -> Result<Vec<Activity>, ActivityError> {
        let r = TypesProtocol::active_list(session, (self.id(), self.class_id()))?;
        let class_ended = self.class_ended();
        let activities = Arc::new(Mutex::new(Vec::new()));
        if let Some(data) = r.data() {
            let thread_count = 1;
            let len = data.active_list().len();
            let chunk_rest = len % thread_count;
            let chunk_count = len / thread_count + if chunk_rest == 0 { 0 } else { 1 };
            for i in 0..chunk_count {
                let ars = &data.active_list()[i * thread_count..if i != chunk_count - 1 {
                    (i + 1) * thread_count
                } else {
                    len
                }];
                let mut handles = Vec::new();
                for ar in ars {
                    let activity_raw = ar.clone();
                    let course_with_info = self.clone();
                    let activities = activities.clone();
                    let handle = std::thread::spawn(move || {
                        if let Some(oid) = activity_raw.other_id.as_ref()
                            && let Ok(other_id) = oid.parse::<i64>()
                            && { (0..=5).contains(&other_id) }
                        {
                            let active_id: String = activity_raw.id.to_string();
                            let base_sign = RawSign::new(
                                active_id,
                                course_with_info.clone(),
                                activity_raw.name_one,
                                other_id,
                                activity_raw.status,
                                activity_raw.start_time_mills.some(),
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
                                    other_id: activity_raw.other_id,
                                    id: activity_raw.id.to_string(),
                                    name: activity_raw.name_one,
                                    course: course_with_info.clone(),
                                    status_code: activity_raw.status,
                                    start_time_mills: activity_raw.start_time_mills.some(),
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
    #[inline]
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
    #[inline]
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

    #[inline]
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
    #[inline]
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
    #[inline]
    pub fn ended(&self) -> bool {
        self.ended
    }
    #[inline]
    pub fn teacher(&self) -> &String {
        &self.teacher
    }
    #[inline]
    pub fn image_url(&self) -> Option<&String> {
        self.image_url.as_ref()
    }
    #[inline]
    pub fn name(&self) -> &String {
        &self.name
    }
}
impl Deref for CourseWithInfo {
    type Target = Course;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.course
    }
}
