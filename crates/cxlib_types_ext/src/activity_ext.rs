use cxlib_error::{CxlibResultUtils, MaybeFatalError};
use cxlib_types::{Activity, Course, Session};
use log::{debug, error, warn};
use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, RecvError},
        Arc,
    },
    time::{Duration, SystemTime},
};

/// 顺序优化。对课程排序，以便更快地获取有效签到。
pub trait CourseCacheSortTrait {
    /// 对课程排序。要求更有可能存在有效签到的课程排在前面。
    fn sort_courses(&self, courses: Vec<Course>) -> Vec<Course>;
}
/// # CourseExcludeInfoTrait
/// 课程排除列表特型。在获取[活动](Activity)列表时排除部分课程的活动，以此提高加载速度。
pub trait CourseExcludeInfoTrait: Send + Sync {
    /// 课程是否被排除，参数为课程 ID.
    fn is_excluded(&self, id: i64) -> bool;
    /// 获取所有被排除的课程的 ID
    fn excluded_courses(&self) -> HashSet<i64>;
    /// 排除某课程，参数为课程 ID.
    fn exclude(&self, id: i64);
    /// 取消对某课程的排除，参数为课程 ID.
    fn cancel(&self, id: i64);
    /// 更新排除列表，参数为课程 ID 的列表。
    /// 在默认实现中，该函数会完全删除旧数据，并更新为新数据。
    fn update<'a, I: IntoIterator<Item = &'a i64>>(&self, excludes: I);
    fn exclude_inactive_course<'a, I: IntoIterator<Item = &'a Activity>>(
        &self,
        course: &Course,
        activities: I,
        // 发布到过期的天数。
        expiry_days: u64,
    ) {
        fn time_delta_from_mills(mills: u64) -> Duration {
            let start_time = std::time::UNIX_EPOCH + Duration::from_millis(mills);
            let now = SystemTime::now();
            now.duration_since(start_time).log_unwrap()
        }
        // 是否存在未过期的签到。
        fn has_unexpired_signs<'a, I: IntoIterator<Item = &'a Activity>>(
            activities: I,
            // 发布到过期的天数。
            expiry_days: u64,
        ) -> bool {
            for activity in activities {
                if let Activity::RawSign(sign) = activity {
                    if time_delta_from_mills(sign.start_time_mills).as_secs() / (24 * 60 * 60)
                        < expiry_days
                    {
                        return true;
                    }
                }
            }
            false
        }
        let id = course.id();
        let dont_exclude = has_unexpired_signs(activities, expiry_days);
        let excluded = self.is_excluded(id);
        if dont_exclude && excluded {
            self.cancel(id);
        } else if !dont_exclude && !excluded {
            self.exclude(id);
        }
    }
}

pub struct ActivitiesReceiver {
    receiver: Receiver<(Vec<Activity>, Course)>,
    sessions: Arc<HashMap<Course, Vec<Session>>>,
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
                    self.sessions = self.receiver.sessions[&course].clone();
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
    fn courses_chunks<'a, Iter: Iterator<Item = &'a Course>>(
        courses: Iter,
        chunk_count: usize,
    ) -> Vec<Vec<&'a Course>> {
        let mut chunks = vec![vec![]; chunk_count];
        for (index, course) in courses.enumerate() {
            chunks[index % chunk_count].push(course);
        }
        chunks
    }
    /// 异步获取指定课程集合的活动。内部使用 [`mpsc::channel`] 实现。
    fn get_from_courses(courses: HashMap<Course, Vec<Session>>) -> ActivitiesReceiver {
        let course_sessions_map = Arc::new(courses);
        let (sender, receiver) = std::sync::mpsc::channel();
        let fatal_error_occurred = Arc::new(AtomicBool::new(false));
        let chunks = Self::courses_chunks(course_sessions_map.keys(), 256);
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
                    debug!("加载课程{course}的签到。");
                    if let Some(session) = course_sessions_map[&course].first() {
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
                        warn!("无法获取用户会话。");
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
