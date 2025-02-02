use crate::ProtocolItem;
use cxlib_error::AgentError;
use ureq::{http::Response, Agent, Body};

// 获取课程
pub fn back_clazz_data(client: &Agent) -> Result<Response<Body>, AgentError> {
    Ok(client
        .get(&format!("{}?view=json&rss=1", ProtocolItem::BackClazzData,))
        .call()?)
}
