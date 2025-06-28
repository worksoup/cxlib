use crate::ClassExt;
use cxlib_protocol::collect::UserProtocolTrait;
use cxlib_types::{Class, CourseError, CourseWithInfo, Session};
use std::collections::HashMap;

pub trait CourseExt {
    /// 通过用户会话获取班级列表。
    ///
    /// # Errors
    /// 仅返回当前语境下的致命错误。
    fn get_from_sessions<
        'a,
        UserProtocol: UserProtocolTrait + Send + 'static,
        Sessions: Iterator<Item = &'a Session<UserProtocol>>,
    >(
        sessions: Sessions,
    ) -> Result<HashMap<CourseWithInfo, Vec<Session<UserProtocol>>>, CourseError> {
        let classes = Class::get_from_sessions(sessions)?;
        Ok(classes
            .into_iter()
            .flat_map(|(k, v)| {
                k.into_courses()
                    .into_iter()
                    .map(move |course| (course, v.clone()))
            })
            .collect())
    }
}
impl CourseExt for CourseWithInfo {}
