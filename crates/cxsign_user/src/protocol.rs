use cxsign_protocol::Protocol;
use ureq::{Agent, Response};

// 账号设置页
pub fn account_manage(client: &Agent) -> Result<Response, Box<ureq::Error>> {
    Ok(client.get(&Protocol::AccountManage.to_string()).call()?)
}
