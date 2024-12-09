use cxlib_error::AgentError;
use cxlib_protocol::ProtocolItem;
use ureq::{Agent, Response};

// 登录页
pub fn login_page(client: &Agent) -> Result<Response, AgentError> {
    Ok(client.get(&ProtocolItem::LoginPage.to_string()).call()?)
}

// 非明文密码登录
pub fn login_enc(client: &Agent, uname: &str, pwd_enc: &str) -> Result<Response, AgentError> {
    Ok(client
        .post(&ProtocolItem::LoginEnc.to_string())
        .set("Content-Type", "application/x-www-form-urlencoded")
        .set("X-Requested-With", "XMLHttpRequest")
        .send_string(&format!("uname={uname}&password={pwd_enc}&fid=-1&t=true&refer=https%253A%252F%252Fi.chaoxing.com&forbidotherlogin=0&validate="))?)
}

// 账号设置页
pub fn account_manage(client: &Agent) -> Result<Response, AgentError> {
    Ok(client
        .get(&ProtocolItem::AccountManage.to_string())
        .call()?)
}
