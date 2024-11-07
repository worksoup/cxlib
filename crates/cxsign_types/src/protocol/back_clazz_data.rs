use cxsign_protocol::ProtocolEnum;
use ureq::{Agent, Response};

// 获取课程
pub fn back_clazz_data(client: &Agent) -> Result<Response, Box<ureq::Error>> {
    Ok(client
        .get(&format!(
            "{}?view=json&rss=1",
            ProtocolEnum::BackClazzData,
        ))
        .call()?)
}
