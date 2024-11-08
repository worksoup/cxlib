use cxsign_protocol::Protocol;
use log::debug;
use ureq::{Agent, Response};

// 签到信息获取
pub fn sign_detail(client: &Agent, active_id: &str) -> Result<Response, Box<ureq::Error>> {
    let url = format!(
        "{}?activePrimaryId={active_id}&type=1",
        Protocol::SignDetail
    );
    debug!("{url}");
    Ok(client.get(&url).call()?)
}
