use crate::ProtocolItem;
use ureq::{Agent, Response};
use cxlib_error::AgentError;

// 获取位置信息列表
pub fn get_location_log(
    session: &Agent,
    (course_id, class_id): (i64, i64),
) -> Result<Response, AgentError> {
    Ok(session
        .get(&format!(
            "{}?DB_STRATEGY=COURSEID&STRATEGY_PARA=courseId&courseId={}&classId={}",
            ProtocolItem::GetLocationLog,
            course_id,
            class_id
        ))
        .call()?)
}
