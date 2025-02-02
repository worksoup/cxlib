use crate::ProtocolItem;
use cxlib_error::AgentError;
use log::debug;
use ureq::{http::Response, Agent, Body};

/// 查询课程活动。
pub fn active_list(
    client: &Agent,
    (course_id, class_id): (i64, i64),
) -> Result<Response<Body>, AgentError> {
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string();
    let url = format!(
        "{}?fid=0&courseId={}&classId={}&showNotStartedActive=0&_={time}",
        ProtocolItem::ActiveList,
        course_id,
        class_id,
    );
    debug!("{url}");
    Ok(client.get(&url).call()?)
}
