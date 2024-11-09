use crate::course::Course;
use cxsign_protocol::ProtocolItem;
use ureq::{Agent, Response};

// 获取位置信息列表
pub fn get_location_log(session: &Agent, course: &Course) -> Result<Response, Box<ureq::Error>> {
    Ok(session
        .get(&format!(
            "{}?DB_STRATEGY=COURSEID&STRATEGY_PARA=courseId&courseId={}&classId={}",
            ProtocolItem::GetLocationLog,
            course.get_id(),
            course.get_class_id()
        ))
        .call()?)
}
