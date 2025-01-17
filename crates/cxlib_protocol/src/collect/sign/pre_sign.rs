use crate::ProtocolItem;
use cxlib_error::AgentError;
use ureq::{Agent, Response};

// 预签到
pub fn pre_sign(
    client: &Agent,
    (course_id, class_id): (i64, i64),
    active_id: &str,
    uid: &str,
) -> Result<Response, AgentError> {
    let url = ProtocolItem::PreSign;
    let url =
        format!("{url}?courseId={course_id}&classId={class_id}&activePrimaryId={active_id}&general=1&sys=1&ls=1&appType=15&&tid=&uid={uid}&ut=s&isTeacherViewOpen=0");
    Ok(client.get(&url).call()?)
}
pub fn pre_sign_for_qrcode_sign(
    client: &Agent,
    (course_id, class_id): (i64, i64),
    active_id: &str,
    uid: &str,
    c: &str,
    enc: &str,
) -> Result<Response, AgentError> {
    let url =
        format!("{}?courseId={course_id}&classId={class_id}&activePrimaryId={active_id}&general=1&sys=1&ls=1&appType=15&&tid=&uid={uid}&ut=s&isTeacherViewOpen=0&rcode={}", ProtocolItem::PreSign, format_args!(
            "&rcode={}",percent_encoding::utf8_percent_encode(&format!("SIGNIN:aid={active_id}&source=15&Code={c}&enc={enc}"), percent_encoding::NON_ALPHANUMERIC)
        ));
    Ok(client.get(&url).call()?)
}
