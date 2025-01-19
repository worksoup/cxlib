use crate::ProtocolItem;
use cxlib_error::AgentError;
use log::debug;
use ureq::{Agent, Response};

// 签到信息获取
pub fn sign_detail(client: &Agent, active_id: &str) -> Result<Response, AgentError> {
    let url = format!(
        "{}?activePrimaryId={active_id}&type=1",
        ProtocolItem::SignDetail
    );
    debug!("{url}");
    Ok(client.get(&url).call()?)
}
