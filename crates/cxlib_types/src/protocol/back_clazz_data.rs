use cxlib_error::AgentError;
use cxlib_protocol::ProtocolItem;
use ureq::{Agent, Response};

// 获取课程
pub fn back_clazz_data(client: &Agent) -> Result<Response, AgentError> {
    Ok(client
        .get(&format!("{}?view=json&rss=1", ProtocolItem::BackClazzData,))
        .call()?)
}
