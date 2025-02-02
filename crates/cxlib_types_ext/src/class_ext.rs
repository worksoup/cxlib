use cxlib_error::{CourseError, MaybeFatalError};
use cxlib_types::{Class, Session};
use log::warn;
use std::collections::{hash_map::Entry, HashMap};

pub trait ClassExt {
    fn get_from_sessions<'a, Sessions: Iterator<Item = &'a Session>>(
        sessions: Sessions,
    ) -> Result<HashMap<Class, Vec<Session>>, CourseError> {
        let mut handles = Vec::new();
        for session in sessions {
            let session_ = session.clone();
            let handle = std::thread::spawn(move || -> Result<Vec<Class>, CourseError> {
                session_.get_classes()
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
