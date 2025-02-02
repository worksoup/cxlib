use cxlib_error::MaybeFatalError;
use cxlib_types::{Activity, Course, Session};
use log::{debug, error, warn};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, RecvError},
        Arc,
    },
};

pub struct ActivitiesReceiver {
    receiver: Receiver<(Vec<Activity>, Course)>,
    sessions: Arc<HashMap<i64, Vec<Session>>>,
}
impl ActivitiesReceiver {
    #[inline]
    pub fn recv(&self) -> Result<(Vec<Activity>, Course), RecvError> {
        self.receiver.recv()
    }
}
pub struct AsyncActivitiesIterator {
    activities: Vec<Activity>,
    sessions: Vec<Session>,
    receiver: ActivitiesReceiver,
}
impl Iterator for AsyncActivitiesIterator {
    type Item = (Activity, Vec<Session>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.activities.is_empty() {
            while let Ok((activities, course)) = self.receiver.recv() {
                if activities.is_empty() {
                    continue;
                } else {
                    self.activities = activities;
                    self.sessions = self.receiver.sessions[&course.id()].clone();
                    return Some((self.activities.remove(0), self.sessions.clone()));
                }
            }
            None
        } else {
            Some((self.activities.remove(0), self.sessions.clone()))
        }
    }
}
impl IntoIterator for ActivitiesReceiver {
    type Item = (Activity, Vec<Session>);
    type IntoIter = AsyncActivitiesIterator;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        AsyncActivitiesIterator {
            activities: vec![],
            sessions: vec![],
            receiver: self,
        }
    }
}
pub trait ActivityExt {
    /// 分块，以便多个线程一同处理。
    #[inline]
    fn courses_chunks<Iter: Iterator>(courses: Iter, chunk_count: usize) -> Vec<Vec<Iter::Item>>
    where
        <Iter as Iterator>::Item: Clone,
    {
        let mut chunks = vec![vec![]; chunk_count];
        for (index, course) in courses.enumerate() {
            chunks[index % chunk_count].push(course);
        }
        chunks
    }
    /// 异步获取指定课程集合的活动。内部使用 [`mpsc::channel`] 实现。
    fn get_from_courses<'a>(
        sorted_courses: impl Iterator<Item = &'a Course>,
        sessions: HashMap<i64, Vec<Session>>,
    ) -> ActivitiesReceiver {
        let course_sessions_map = Arc::new(sessions);
        let (sender, receiver) = std::sync::mpsc::channel();
        let fatal_error_occurred = Arc::new(AtomicBool::new(false));
        let chunks = Self::courses_chunks(sorted_courses.into_iter(), 256);
        for courses in chunks {
            let fatal_error_occurred = Arc::clone(&fatal_error_occurred);
            let sender = sender.clone();
            let course_sessions_map = Arc::clone(&course_sessions_map);
            let courses = courses.into_iter().cloned().collect::<Vec<_>>();
            std::thread::spawn(move || {
                for course in courses {
                    if fatal_error_occurred.load(Ordering::Relaxed) {
                        break;
                    }
                    debug!("加载课程 [{course}] 的签到。");
                    if let Some(session) = course_sessions_map[&course.id()].first() {
                        let activities = course.get_activities(session);
                        match activities {
                            Ok(activities) => match sender.send((activities, course)) {
                                Ok(_) => {}
                                Err(e) => {
                                    warn!("Receiver is dropped: `{e}`.",);
                                    fatal_error_occurred.store(true, Ordering::Relaxed);
                                    return;
                                }
                            },
                            Err(e) => {
                                if e.is_fatal() {
                                    error!("`{e}`.");
                                    fatal_error_occurred.store(true, Ordering::Relaxed);
                                    return;
                                } else {
                                    warn!("`{e}`.");
                                    continue;
                                }
                            }
                        }
                    } else {
                        warn!("无法获取课程 [{course}] 的用户会话。",);
                    }
                }
                drop(sender);
            });
        }
        ActivitiesReceiver {
            receiver,
            sessions: course_sessions_map,
        }
    }
}
impl ActivityExt for Activity {}
