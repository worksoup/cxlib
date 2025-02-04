use cxlib_error::MaybeFatalError;
use cxlib_types::{Activity, Course, Session};
use log::{debug, error, warn};
use std::{
    mem,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, RecvError},
        Arc,
    },
};

pub struct ActivitiesReceiver {
    receiver: Receiver<(Vec<Activity>, Course, Vec<Session>)>,
}
impl ActivitiesReceiver {
    #[inline]
    pub fn recv(&self) -> Result<(Vec<Activity>, Course, Vec<Session>), RecvError> {
        self.receiver.recv()
    }
}
pub struct AsyncActivitiesIterator {
    activities: Vec<Activity>,
    course: Course,
    sessions: Vec<Session>,
    receiver: ActivitiesReceiver,
}
impl Iterator for AsyncActivitiesIterator {
    type Item = (Activity, Course, Vec<Session>);

    fn next(&mut self) -> Option<Self::Item> {
        while self.activities.is_empty() {
            if let Ok((activities, course, sessions)) = self.receiver.recv() {
                self.activities = activities;
                self.course = course;
                self.sessions = sessions;
                continue;
            }
            return None;
        }
        let (course, sessions) = if self.activities.len() == 1 {
            (self.course.take(), mem::take(&mut self.sessions))
        } else {
            (self.course.clone(), self.sessions.clone())
        };
        let activities = self.activities.remove(0);
        Some((activities, course, sessions))
    }
}
impl IntoIterator for ActivitiesReceiver {
    type Item = (Activity, Course, Vec<Session>);
    type IntoIter = AsyncActivitiesIterator;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        AsyncActivitiesIterator {
            activities: vec![],
            course: Course::none(),
            sessions: vec![],
            receiver: self,
        }
    }
}
pub trait ActivityExt {
    /// 分块，以便多个线程一同处理。
    #[inline]
    fn courses_chunks<Iter: Iterator>(courses: Iter, chunk_size: usize) -> Vec<Vec<Iter::Item>>
    where
        <Iter as Iterator>::Item: Clone,
    {
        let mut chunks = vec![vec![]; chunk_size];
        for (index, course) in courses.enumerate() {
            chunks[index % chunk_size].push(course);
        }
        chunks
    }
    /// 异步获取指定课程集合的活动。内部使用 [`mpsc::channel`] 实现。
    fn get_from_courses(
        sorted_courses: impl Iterator<Item = (Course, Vec<Session>)>,
    ) -> ActivitiesReceiver {
        let (sender, receiver) = std::sync::mpsc::channel();
        let fatal_error_occurred = Arc::new(AtomicBool::new(false));
        let chunks = Self::courses_chunks(sorted_courses, 256);
        for courses in chunks {
            let fatal_error_occurred = Arc::clone(&fatal_error_occurred);
            let sender = sender.clone();
            std::thread::spawn(move || {
                for (course, sessions) in courses {
                    if fatal_error_occurred.load(Ordering::Relaxed) {
                        break;
                    }
                    debug!("加载课程 [{course}] 的签到。");
                    if let Some(session) = sessions.first() {
                        let activities = course.get_activities(session);
                        match activities {
                            Ok(activities) => match sender.send((activities, course, sessions)) {
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
        ActivitiesReceiver { receiver }
    }
}
impl ActivityExt for Activity {}
