use crate::error::ActivityError;
use crate::{RawSign, Session, UnhandledGeoAddrWithRange};
use cxlib_base_types::{Activity, CourseWithInfo, OtherActivity};
use cxlib_error::AgentError;
use cxlib_protocol::collect::{TypesProtocolTrait, UserProtocolTrait};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use ureq::Agent;
pub trait CourseWithInfoExt {
    fn get_locations<TypesProtocol>(
        &self,
        session: &Agent,
    ) -> Result<HashMap<String, UnhandledGeoAddrWithRange>, AgentError>
    where
        TypesProtocol: TypesProtocolTrait;
    fn get_activities<TypesProtocol: TypesProtocolTrait, UserProtocol: UserProtocolTrait>(
        &self,
        session: &Session<UserProtocol>,
    ) -> Result<Vec<Activity>, ActivityError>;
}

impl CourseWithInfoExt for CourseWithInfo {
    // TODO: 该函数需要注意：API 可能已经失效。
    #[inline]
    fn get_locations<TypesProtocol>(
        &self,
        session: &Agent,
    ) -> Result<HashMap<String, UnhandledGeoAddrWithRange>, AgentError>
    where
        TypesProtocol: TypesProtocolTrait,
    {
        let r = TypesProtocol::get_location_log(session, (self.id(), self.class_id()))?;
        let mut map = HashMap::new();
        for l in r.data() {
            map.insert(l.active_id().to_string(), l.to_location_with_range());
        }
        Ok(map)
    }
    /// 获取该课程的活动。
    fn get_activities<TypesProtocol: TypesProtocolTrait, UserProtocol: UserProtocolTrait>(
        &self,
        session: &Session<UserProtocol>,
    ) -> Result<Vec<Activity>, ActivityError> {
        let r = TypesProtocol::active_list(session, (self.id(), self.class_id()))?;
        let class_ended = self.class_ended();
        let activities = Arc::new(Mutex::new(Vec::new()));
        if let Some(data) = r.data() {
            let thread_count = 1;
            let len = data.active_list().len();
            let chunk_rest = len % thread_count;
            let chunk_count = len / thread_count + if chunk_rest == 0 { 0 } else { 1 };
            for i in 0..chunk_count {
                let ars = &data.active_list()[i * thread_count..if i != chunk_count - 1 {
                    (i + 1) * thread_count
                } else {
                    len
                }];
                let mut handles = Vec::new();
                for ar in ars {
                    let activity_raw = ar.clone();
                    let course_with_info = self.clone();
                    let activities = activities.clone();
                    let handle = std::thread::spawn(move || {
                        if let Some(oid) = activity_raw.other_id.as_ref()
                            && let Ok(other_id) = oid.parse::<i64>()
                            && { (0..=5).contains(&other_id) }
                        {
                            let active_id: String = activity_raw.id.to_string();
                            let base_sign = RawSign::new(
                                active_id,
                                course_with_info.clone(),
                                activity_raw.name_one,
                                other_id,
                                activity_raw.status,
                                activity_raw.start_time_mills.some(),
                                class_ended,
                            );
                            activities
                                .lock()
                                .unwrap()
                                .push(Activity::RawSign(base_sign))
                        } else {
                            activities
                                .lock()
                                .unwrap()
                                .push(Activity::Other(OtherActivity {
                                    other_id: activity_raw.other_id,
                                    id: activity_raw.id.to_string(),
                                    name: activity_raw.name_one,
                                    course: course_with_info.clone(),
                                    status_code: activity_raw.status,
                                    start_time_mills: activity_raw.start_time_mills.some(),
                                }))
                        }
                    });
                    handles.push(handle);
                }
                for h in handles {
                    h.join().unwrap();
                }
            }
        }
        let activities = Arc::into_inner(activities).unwrap().into_inner().unwrap();
        Ok(activities)
    }
}
