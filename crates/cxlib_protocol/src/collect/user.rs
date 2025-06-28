use cxlib_error::AgentError;
use ureq::{Agent, Body, http::Response};

pub trait UserProtocolTrait {
    fn login_page_url() -> &'static str {
        UserProtocol::LOGIN_PAGE
    }
    fn login_enc_url() -> &'static str {
        UserProtocol::LOGIN_ENC
    }
    fn account_manage_url() -> &'static str {
        UserProtocol::ACCOUNT_MANAGE
    }
    fn back_clazz_data_url() -> &'static str {
        UserProtocol::BACK_CLAZZ_DATA
    }
    // 登录页
    fn login_page(client: &Agent) -> Result<Response<Body>, AgentError> {
        let url = Self::login_page_url();
        Ok(client.get(url).call()?)
    }

    // 非明文密码登录
    fn login_enc(client: &Agent, uname: &str, pwd_enc: &str) -> Result<Response<Body>, AgentError> {
        let url = Self::login_enc_url();
        Ok(client
            .post(url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("X-Requested-With", "XMLHttpRequest")
            .send(format!("uname={uname}&password={pwd_enc}&fid=-1&t=true&refer=https%253A%252F%252Fi.chaoxing.com&forbidotherlogin=0&validate="))?)
    }

    // 账号设置页
    fn account_manage(client: &Agent) -> Result<Response<Body>, AgentError> {
        let url = Self::account_manage_url();
        Ok(client.get(url).call()?)
    }

    // 获取课程
    fn back_clazz_data(client: &Agent) -> Result<Response<Body>, AgentError> {
        let url = Self::back_clazz_data_url();
        Ok(client.get(&format!("{url}?view=json&rss=1")).call()?)
    }
}
pub struct UserProtocol;
impl UserProtocol {
    /// 登录页
    pub const LOGIN_PAGE: &'static str = "https://passport2.chaoxing.com/mlogin?fid=&newversion=true&refer=http%3A%2F%2Fi.chaoxing.com";
    /// 非明文密码登录
    pub const LOGIN_ENC: &'static str = "https://passport2.chaoxing.com/fanyalogin";
    /// 账号设置页
    pub const ACCOUNT_MANAGE: &'static str = "https://passport2.chaoxing.com/mooc/accountManage";
    /// 获取课程
    pub const BACK_CLAZZ_DATA: &'static str =
        "https://mooc1-api.chaoxing.com/mycourse/backclazzdata";
}
impl UserProtocolTrait for UserProtocol {}
