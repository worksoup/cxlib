use cxsign_protocol::ProtocolItem;
use cxsign_types::Course;
use log::debug;
use ureq::{Agent, Response};

/// 查询课程活动。
pub fn active_list(client: &Agent, course: Course) -> Result<Response, Box<ureq::Error>> {
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string();
    let url = format!(
        "{}?fid=0&courseId={}&classId={}&showNotStartedActive=0&_={time}",
        ProtocolItem::ActiveList,
        course.get_id(),
        course.get_class_id(),
    );
    debug!("{url}");
    Ok(client.get(&url).call()?)
}
