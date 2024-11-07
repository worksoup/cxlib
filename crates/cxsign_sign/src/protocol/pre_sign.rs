use cxsign_protocol::ProtocolEnum;
use cxsign_types::Course;
use ureq::{Agent, Response};

// 预签到
pub fn pre_sign(
    client: &Agent,
    course: Course,
    active_id: &str,
    uid: &str,
) -> Result<Response, Box<ureq::Error>> {
    let course_id = course.get_id();
    let class_id = course.get_class_id();
    let url = ProtocolEnum::PreSign;
    let url =
        format!("{url}?courseId={course_id}&classId={class_id}&activePrimaryId={active_id}&general=1&sys=1&ls=1&appType=15&&tid=&uid={uid}&ut=s&isTeacherViewOpen=0");
    Ok(client.get(&url).call()?)
}
pub fn pre_sign_for_qrcode_sign(
    client: &Agent,
    course: Course,
    active_id: &str,
    uid: &str,
    c: &str,
    enc: &str,
) -> Result<Response, Box<ureq::Error>> {
    let course_id = course.get_id();
    let class_id = course.get_class_id();
    let url =
        format!("{}?courseId={course_id}&classId={class_id}&activePrimaryId={active_id}&general=1&sys=1&ls=1&appType=15&&tid=&uid={uid}&ut=s&isTeacherViewOpen=0&rcode={}", ProtocolEnum::PreSign, format_args!(
            "&rcode={}",percent_encoding::utf8_percent_encode(&format!("SIGNIN:aid={active_id}&source=15&Code={c}&enc={enc}"), percent_encoding::NON_ALPHANUMERIC)
        ));
    Ok(client.get(&url).call()?)
}
