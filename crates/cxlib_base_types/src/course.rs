use crate::{ClassInfo, RawCourse};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, write},
    ops::Deref,
    str::FromStr,
};

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
