use crate::CourseWithInfo;
use bincode::{Decode, Encode};
use getset2::Getset2;
use serde::{Deserialize, Serialize};

/// # RawSign
///
/// 未分类的课程签到。
///
/// 对于该类型的分类、处理等，请参考 `cxlib_default_impl::sign` 中的相关部分。
#[derive(
    Debug,
    PartialEq,
    PartialOrd,
    Ord,
    Eq,
    Hash,
    Clone,
    Serialize,
    Deserialize,
    Getset2,
    Decode,
    Encode,
)]
#[getset2(get_ref(pub))]
pub struct RawSign {
    active_id: String,
    course: CourseWithInfo,
    name: String,
    other_id: i64,
    #[getset2(set(pub))]
    status_code: i32,
    start_time_mills: Option<u64>,
    class_ended: bool,
}
impl RawSign {
    #[inline]
    pub fn new(
        active_id: String,
        course: CourseWithInfo,
        name: String,
        other_id: i64,
        status_code: i32,
        start_time_mills: Option<u64>,
        class_ended: bool,
    ) -> Self {
        Self {
            start_time_mills,
            active_id,
            name,
            course,
            other_id,
            status_code,
            class_ended,
        }
    }
}
