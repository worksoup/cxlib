use cxlib_error::AgentError;
use ureq::{Agent, Response};
use crate::ProtocolItem;

// 获取签到之后的信息，例如签到时的 ip, UA, 时间等
// 参见 "http://mobilelearn.chaoxing.com/page/sign/signIn?courseId=$&classId=$&activeId=$&fid=$"
pub fn get_attend_info(client: &Agent, active_id: &str) -> Result<Response, AgentError> {
    Ok(client
        .get(&format!(
            "{}?activeId={active_id}&type=1",
            ProtocolItem::GetAttendInfo
        ))
        .call()?)
}
