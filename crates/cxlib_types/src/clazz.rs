use crate::{Course, RawCourse};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
#[derive(Copy, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ClassId {
    Id(i64),
    /// 如果该班级为用户自建班级，则为此变体。
    TeacherId(i64),
}
impl Display for ClassId {
    #[inline]
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClassId::Id(id) => id.fmt(fmt),
            ClassId::TeacherId(tea_id) => tea_id.fmt(fmt),
        }
    }
}
impl From<ClassId> for i64 {
    #[inline]
    fn from(value: ClassId) -> Self {
        match value {
            ClassId::Id(id) => id,
            ClassId::TeacherId(id) => id,
        }
    }
}
impl ClassId {}

/// # [`ClassInfo`]
/// 班级信息，包括班级 ID 以及是否结课。
#[derive(Copy, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ClassInfo {
    id: ClassId,
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
    pub fn new(id: ClassId, ended: bool) -> ClassInfo {
        ClassInfo { id, ended }
    }
    /// 返回 [`ClassId`].
    #[inline]
    pub fn id(&self) -> ClassId {
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
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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
    pub fn into_courses(self) -> Vec<Course> {
        let Self { raw_courses, info } = self;
        raw_courses
            .into_iter()
            .map(|c| c.into_course(info))
            .collect()
    }
}
