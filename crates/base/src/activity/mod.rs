pub mod sign;

use crate::activity::sign::Sign;
use crate::course::Course;
use crate::user::session::Session;
use serde::{Deserialize, Serialize};
use sign::SignTrait;

#[derive(Debug)]
pub enum Activity {
    Sign(Sign),
    Other(OtherActivity),
}

impl Activity {
    pub fn get_list_from_course(
        签到会话: &Session,
        c: &Course,
    ) -> Result<Vec<Activity>, ureq::Error> {
        let r = crate::protocol::active_list(签到会话, c.clone())?;
        let r: GetActivityR = r.into_json().unwrap();
        let mut 活动列表 = Vec::new();
        if let Some(data) = r.data {
            for ar in data.activeList {
                if let Some(other_id) = ar.otherId {
                    let other_id_i64: i64 = other_id.parse().unwrap();
                    if (0..=5).contains(&other_id_i64) {
                        let 活动id = ar.id.to_string();
                        let detail = sign::BaseSign::通过active_id获取签到信息(
                            活动id.as_str(),
                            签到会话,
                        )?;
                        let base_sign = sign::BaseSign {
                            活动id,
                            签到名: ar.nameOne,
                            课程: c.clone(),
                            other_id,
                            状态码: ar.status,
                            开始时间戳: (ar.startTime / 1000) as i64,
                            签到信息: detail,
                        };
                        活动列表.push(Activity::Sign(base_sign.to_sign()))
                    } else {
                        活动列表.push(Activity::Other(OtherActivity {
                            id: ar.id.to_string(),
                            name: ar.nameOne,
                            course: c.clone(),
                            status: ar.status,
                            start_time_secs: (ar.startTime / 1000) as i64,
                        }))
                    }
                } else {
                    活动列表.push(Activity::Other(OtherActivity {
                        id: ar.id.to_string(),
                        name: ar.nameOne,
                        course: c.clone(),
                        status: ar.status,
                        start_time_secs: (ar.startTime / 1000) as i64,
                    }))
                }
            }
        }
        Ok(活动列表)
    }
}

#[derive(Debug)]
pub struct OtherActivity {
    pub id: String,
    pub name: String,
    pub course: Course,
    pub status: i32,
    pub start_time_secs: i64,
}

#[derive(Deserialize, Serialize)]
#[allow(non_snake_case)]
struct ActivityRaw {
    nameOne: String,
    id: i64,
    otherId: Option<String>,
    status: i32,
    startTime: u64,
}

#[derive(Deserialize, Serialize)]
#[allow(non_snake_case)]
struct Data {
    activeList: Vec<ActivityRaw>,
}

#[derive(Deserialize, Serialize)]
struct GetActivityR {
    data: Option<Data>,
}
