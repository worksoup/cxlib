use cxlib_error::MaybeFatalError;
use cxlib_types::{Activity, Course, Session};
use log::{debug, error, warn};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{Receiver, RecvError},
    Arc,
};
pub struct ActivitiesReceiver {
    receiver: Receiver<(Vec<Activity>, Vec<Session>)>,
}
impl ActivitiesReceiver {
    #[inline]
    pub fn recv(&self) -> Result<(Vec<Activity>, Vec<Session>), RecvError> {
        self.receiver.recv()
    }
}
impl IntoIterator for ActivitiesReceiver {
    type Item = (Vec<Activity>, Vec<Session>);
    type IntoIter = <Receiver<(Vec<Activity>, Vec<Session>)> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.receiver.into_iter()
    }
}
impl<'a> IntoIterator for &'a ActivitiesReceiver {
    type Item = (Vec<Activity>, Vec<Session>);
    type IntoIter = <&'a Receiver<(Vec<Activity>, Vec<Session>)> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.receiver.iter()
    }
}
pub trait ActivityExt {
    /// 分块，以便多个线程一同处理。
    #[inline]
    fn courses_chunks<Iter: Iterator>(
        courses: Iter,
        max_chunk_size: usize,
    ) -> Vec<Vec<Iter::Item>> {
        let mut chunks: Vec<Vec<_>> = Vec::new();
        for (index, course) in courses.enumerate() {
            if let Some(chunk) = chunks.get_mut(index % max_chunk_size) {
                chunk.push(course);
            } else {
                chunks.push(vec![course]);
            }
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
                            Ok(activities) => {
                                // 有活动才发送。
                                if !activities.is_empty() {
                                    match sender.send((activities, sessions)) {
                                        Ok(_) => {}
                                        Err(e) => {
                                            warn!("Receiver is dropped: `{e}`.",);
                                            fatal_error_occurred.store(true, Ordering::Relaxed);
                                            return;
                                        }
                                    }
                                }
                            }
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
