use cxsign_protocol::ProtocolEnum;
use ureq::{Agent, Response};

// 账号设置页
pub fn account_manage(client: &Agent) -> Result<Response, Box<ureq::Error>> {
    Ok(client.get(&ProtocolEnum::AccountManage).call()?)
}
