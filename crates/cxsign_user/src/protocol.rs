use cxsign_protocol::ProtocolItem;
use ureq::{Agent, Response};

// 账号设置页
pub fn account_manage(client: &Agent) -> Result<Response, Box<ureq::Error>> {
    Ok(client.get(&ProtocolItem::AccountManage.to_string()).call()?)
}
