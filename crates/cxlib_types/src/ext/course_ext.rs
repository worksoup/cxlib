use crate::ext::ClassExt;
use cxlib_protocol::collect::UserProtocolTrait;
use std::collections::HashMap;

use crate::{Class, CourseError, CourseWithInfo, Session};

pub trait CourseExt {
    /// 通过用户会话获取班级列表。
    ///
    /// # Errors
    /// 仅返回当前语境下的致命错误。
    fn get_from_sessions<
        'a,
        UserProtocol: UserProtocolTrait + Send + 'static,
        Sessions: Iterator<Item = &'a Session>,
    >(
        sessions: Sessions,
    ) -> Result<HashMap<CourseWithInfo, Vec<Session>>, CourseError> {
        let classes = Class::get_from_sessions::<UserProtocol, _>(sessions)?;
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
