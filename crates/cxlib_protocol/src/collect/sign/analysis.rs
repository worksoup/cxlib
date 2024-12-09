use cxlib_error::AgentError;
use ureq::{Agent, Response};
use crate::ProtocolItem;

// analysis
pub fn analysis(client: &Agent, active_id: &str) -> Result<Response, AgentError> {
    let url = ProtocolItem::Analysis;
    let url = format!("{url}?vs=1&DB_STRATEGY=RANDOM&aid={active_id}");
    Ok(client.get(&url).call()?)
}

// analysis 2
pub fn analysis2(client: &Agent, code: &str) -> Result<Response, AgentError> {
    let url = ProtocolItem::Analysis2;
    let url = format!("{url}?DB_STRATEGY=RANDOM&code={code}");
    Ok(client.get(&url).call()?)
}
