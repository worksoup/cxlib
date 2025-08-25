use cxlib_error_utils::MaybeFatalError;
use cxlib_protocol::collect::UserProtocolTrait;
use log::warn;
use std::collections::{HashMap, hash_map::Entry};

use crate::{Class, CourseError, Session};

pub trait ClassExt {
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
    ) -> Result<HashMap<Class, Vec<Session>>, CourseError> {
        // 由于多个线程几乎同时启动，故不需要 fatal_error_occurred 变量判断是否出现致命错误而直接返回。
        let mut handles = Vec::new();
        for session in sessions {
            let session_ = session.clone();
            let handle = std::thread::spawn(move || -> Result<Vec<Class>, CourseError> {
                session_.get_classes::<UserProtocol>()
            });
            handles.push((handle, session));
        }
        let mut classes = HashMap::<_, Vec<_>>::new();
        for (handle, session) in handles {
            let r = handle.join().unwrap();
            let classes_ = match r {
                Ok(c) => c,
                Err(e) => {
                    if e.is_fatal() {
                        return Err(e);
                    } else {
                        warn!("未能获取用户[{}]的课程，错误信息：{e}.", session.name());
                        Default::default()
                    }
                }
            };
            for clazz in classes_ {
                let entry = classes.entry(clazz);
                match entry {
                    Entry::Occupied(mut entry) => {
                        entry.get_mut().push(session.clone());
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(vec![session.clone()]);
                    }
                }
            }
        }
        Ok(classes)
    }
}
impl ClassExt for Class {}
