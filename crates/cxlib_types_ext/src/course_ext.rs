use crate::ClassExt;
use cxlib_error::CourseError;
use cxlib_types::{Class, Course, Session};
use std::collections::HashMap;

pub trait CourseExt {
    fn get_from_sessions<'a, Sessions: Iterator<Item = &'a Session>>(
        sessions: Sessions,
    ) -> Result<HashMap<Course, Vec<Session>>, CourseError> {
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
impl CourseExt for Course {}
