pub use cxlib_error::ActivityError;

use crate::{Course, RawSign};
/// # Activity
///
/// 活动类型，是一个枚举，可能是一个[暂未被分类的课程签到](RawSign)，也可能是[其他活动](OtherActivity)，如通知、作业等。
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub enum Activity {
    RawSign(RawSign),
    Other(OtherActivity),
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

impl Activity {
    pub fn course(&self) -> &Course {
        match self {
            Activity::RawSign(a) => &a.course,
            Activity::Other(a) => &a.course,
        }
    }
    pub fn start_time_mills(&self) -> u64 {
        match self {
            Activity::RawSign(a) => a.start_time_mills,
            Activity::Other(a) => a.start_time_mills,
        }
    }
}
