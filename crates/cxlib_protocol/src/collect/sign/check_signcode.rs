use ureq::{Agent, Response};
use cxlib_error::AgentError;
use crate::ProtocolItem;

// 签到码检查
pub fn check_signcode(
    client: &Agent,
    active_id: &str,
    signcode: &str,
) -> Result<Response, AgentError> {
    Ok(client
        .get(&format!(
            "{}?activeId={active_id}&signCode={signcode}",
            ProtocolItem::CheckSigncode
        ))
        .call()?)
}
