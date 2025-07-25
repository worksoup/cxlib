use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::{CourseWithInfo, RawSign};
/// # Activity
///
/// 活动类型，是一个枚举，可能是一个[暂未被分类的课程签到](RawSign)，也可能是[其他活动](OtherActivity)，如通知、作业等。
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Serialize, Deserialize, Decode, Encode)]
pub enum Activity {
    RawSign(RawSign),
    Other(OtherActivity),
}
/// # OtherActivity
///
/// 除课程签到外的其他活动，如通知、作业等。
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Decode, Encode, Serialize, Deserialize)]
pub struct OtherActivity {
    pub id: String,
    pub name: String,
    pub course: CourseWithInfo,
    pub status: i32,
    pub start_time_mills: u64,
}

impl Activity {
    pub fn id(&self) -> &String {
        match self {
            Activity::RawSign(a) => a.active_id(),
            Activity::Other(a) => &a.id,
        }
    }
    pub fn course(&self) -> &CourseWithInfo {
        match self {
            Activity::RawSign(a) => a.course(),
            Activity::Other(a) => &a.course,
        }
    }
    pub fn start_time_mills(&self) -> u64 {
        match self {
            Activity::RawSign(a) => *a.start_time_mills(),
            Activity::Other(a) => a.start_time_mills,
        }
    }
}
