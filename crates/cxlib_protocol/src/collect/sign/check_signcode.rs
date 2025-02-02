use crate::ProtocolItem;
use cxlib_error::AgentError;
use ureq::{http::Response, Agent, Body};

// 签到码检查
pub fn check_signcode(
    client: &Agent,
    active_id: &str,
    signcode: &str,
) -> Result<Response<Body>, AgentError> {
    Ok(client
        .get(&format!(
            "{}?activeId={active_id}&signCode={signcode}",
            ProtocolItem::CheckSigncode
        ))
        .call()?)
}
