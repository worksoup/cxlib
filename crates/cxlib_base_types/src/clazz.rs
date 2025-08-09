use crate::{CourseWithInfo, RawCourse};
use bincode::{Decode, Encode};
use serde::Serialize;

/// # [`ClassInfo`]
/// 班级信息，包括班级 ID 以及是否结课。
#[derive(Copy, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Decode, Encode)]
pub struct ClassInfo {
    id: i64,
    ended: bool,
}
impl ClassInfo {
    // #[inline]
    // pub fn none() -> Self {
    //     Self {
    //         id: ClassId::Id(-1),
    //         ended: true,
    //     }
    // }
    #[inline]
    pub fn new(id: i64, ended: bool) -> ClassInfo {
        ClassInfo { id, ended }
    }
    #[inline]
    pub fn new_with_state(id: i64, state: Option<u8>) -> ClassInfo {
        let ended = state.is_none_or(|state| state == 1);
        ClassInfo { id, ended }
    }
    /// 返回 [`ClassId`].
    #[inline]
    pub fn id(&self) -> i64 {
        self.id
    }
    /// 返回是否已经结课。
    #[inline]
    pub fn ended(&self) -> bool {
        self.ended
    }
}
/// # [`Class`]
/// 代表班级，通过 [`raw_courses`](Class::raw_courses) 获取班级内的课程（不包含班级信息）。
/// 通过 [`into_courses`](Class::into_courses) 获取班级内的课程（包含班级信息）。
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Class {
    raw_courses: Vec<RawCourse>,
    info: ClassInfo,
}
impl Class {
    #[inline]
    pub fn new(courses: Vec<RawCourse>, info: ClassInfo) -> Self {
        Self {
            raw_courses: courses,
            info,
        }
    }
    /// 获取班级信息。
    #[inline]
    pub fn info(&self) -> &ClassInfo {
        &self.info
    }
    /// 获取班级内的课程（不包含班级信息）。
    #[inline]
    pub fn raw_courses(&self) -> &[RawCourse] {
        &self.raw_courses
    }
    /// 获取班级内的课程（包含班级信息）。
    #[inline]
    pub fn into_courses(self) -> Vec<CourseWithInfo> {
        let Self { raw_courses, info } = self;
        raw_courses
            .into_iter()
            .map(|c| c.into_course(info))
            .collect()
    }
}