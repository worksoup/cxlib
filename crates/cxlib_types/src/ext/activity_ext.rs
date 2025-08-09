use cxlib_error_utils::MaybeFatalError;
use cxlib_protocol::collect::{TypesProtocolTrait, UserProtocolTrait};
use log::{debug, error, warn};
use std::{
    ops::DerefMut,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, RecvError},
    },
};

use crate::{Activity, ActivityError, CourseWithInfo, Session, ext::CourseWithInfoExt};

/// 类型别名，代表接收端所接受数据的类型。
pub type ActivitiesSessionsPair<UserProtocol> = (Vec<Activity>, Vec<Session<UserProtocol>>);
/// 接收端本身的类型。
pub type ActivitiesReceiverInner<UserProtocol> = Receiver<ActivitiesSessionsPair<UserProtocol>>;
#[derive(thiserror::Error, Debug)]
pub enum ActivitiesReceiverError<UserProtocol> {
    #[error(transparent)]
    RecvError(#[from] RecvError),
    #[error(transparent)]
    SendError(#[from] ActivitiesSessionsPair<UserProtocol>),
    #[error(transparent)]
    ActivityError(#[from] ActivityError),
}
/// 接收端的可迭代包装。
///
/// 不推荐使用 for 循环迭代元素。
///
/// 请使用 while 循环与 [`ActivitiesReceiver::recv`], 以获取错误信息。
pub struct ActivitiesReceiver<UserProtocol> {
    receiver: ActivitiesReceiverInner<UserProtocol>,
    fatal_error_occurred: Arc<AtomicBool>,
    fatal_error: Arc<Mutex<Option<ActivitiesReceiverError<UserProtocol>>>>,
}
impl<T> MaybeFatalError for ActivitiesReceiverError<T> {
    /// 当前语境下均为不可恢复错误。
    #[inline]
    fn is_fatal(&self) -> bool {
        true
    }
}
impl<T> ActivitiesReceiver<T> {
    /// # Errors
    /// 返回的错误均为当前语境下不可恢复错误。
    #[inline]
    pub fn recv(&self) -> Result<ActivitiesSessionsPair<T>, ActivitiesReceiverError<T>> {
        if self.fatal_error_occurred.load(Ordering::Relaxed) {
            let mut error = self.fatal_error.lock().unwrap();
            Err(error.deref_mut().take().unwrap())?
        }
        Ok(self.receiver.recv()?)
    }
}
impl<T> IntoIterator for ActivitiesReceiver<T> {
    type Item = (Vec<Activity>, Vec<Session<T>>);
    type IntoIter = <Receiver<(Vec<Activity>, Vec<Session<T>>)> as IntoIterator>::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.receiver.into_iter()
    }
}
impl<'a, T> IntoIterator for &'a ActivitiesReceiver<T> {
    type Item = (Vec<Activity>, Vec<Session<T>>);
    type IntoIter = <&'a Receiver<(Vec<Activity>, Vec<Session<T>>)> as IntoIterator>::IntoIter;

    #[inline]
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
    /// 异步获取指定课程集合的活动。
    /// 通过 [`mpsc::channel`] 实现：多线程获取活动，获取的活动进入 Sender 中，返回值为 Receiver, 可以通过迭代器 API 处理。
    fn get_from_courses<
        TypesProtocol: TypesProtocolTrait,
        UserProtocol: UserProtocolTrait + std::marker::Send + 'static,
    >(
        sorted_courses: impl Iterator<
            Item = (
                CourseWithInfo,
                impl IntoIterator<Item = Session<UserProtocol>>,
            ),
        >,
    ) -> ActivitiesReceiver<UserProtocol> {
        let (sender, receiver) = std::sync::mpsc::channel();
        let fatal_error_occurred = Arc::new(AtomicBool::new(false));
        let fatal_error = Arc::new(Mutex::new(None));
        let chunks = Self::courses_chunks(sorted_courses, 256);
        for courses in chunks {
            // 如果发生了不可恢复错误，就停止所有线程。
            let fatal_error_occurred = Arc::clone(&fatal_error_occurred);
            let fatal_error = Arc::clone(&fatal_error);
            let sender = sender.clone();
            // 如果发生了不可恢复错误，不再开启新一轮多线程网络请求。
            if fatal_error_occurred.load(Ordering::Relaxed) {
                break;
            }
            let courses: Vec<(CourseWithInfo, Vec<Session<UserProtocol>>)> = courses
                .into_iter()
                .map(|(course, sessions)| (course, sessions.into_iter().collect::<Vec<_>>()))
                .collect::<Vec<_>>();
            std::thread::spawn(move || {
                for (course, sessions) in courses {
                    if fatal_error_occurred.load(Ordering::Relaxed) {
                        break;
                    }
                    let sessions = sessions.into_iter().collect::<Vec<_>>();
                    debug!("加载课程 [{course}] 的签到。");
                    if let Some(session) = sessions.first() {
                        let activities = course.get_activities::<TypesProtocol, _>(session);
                        match activities {
                            Ok(activities) => {
                                // 有活动才发送。
                                if !activities.is_empty() {
                                    match sender.send((activities, sessions)) {
                                        Ok(_) => {}
                                        Err(e) => {
                                            warn!("Receiver is dropped: `{e}`.",);
                                            fatal_error_occurred.store(true, Ordering::Relaxed);
                                            fatal_error
                                                .lock()
                                                .unwrap()
                                                .replace(ActivitiesReceiverError::from(e.0));
                                            return;
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                if e.is_fatal() {
                                    error!("`{e}`.");
                                    fatal_error_occurred.store(true, Ordering::Relaxed);
                                    fatal_error
                                        .lock()
                                        .unwrap()
                                        .replace(ActivitiesReceiverError::from(e));
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
            fatal_error_occurred,
            fatal_error,
        }
    }
}
impl ActivityExt for Activity {}
