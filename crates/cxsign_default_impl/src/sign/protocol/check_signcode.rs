use cxsign_protocol::ProtocolEnum;
use ureq::{Agent, Response};

// 签到码检查
pub fn check_signcode(
    client: &Agent,
    active_id: &str,
    signcode: &str,
) -> Result<Response, Box<ureq::Error>> {
    Ok(client
        .get(&format!(
            "{}?activeId={active_id}&signCode={signcode}",
            ProtocolEnum::CheckSigncode
        ))
        .call()?)
}
