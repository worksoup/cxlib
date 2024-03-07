use ureq::{Agent, Response};

// 非明文密码登录
static LOGIN_ENC: &str = "http://passport2.chaoxing.com/fanyalogin";

pub async fn login_enc(
    client: &Agent,
    uname: &str,
    pwd_enc: &str,
) -> Result<Response, ureq::Error> {
    client
        .post(LOGIN_ENC)
        .set("Content-Type", "application/x-www-form-urlencoded")
        .set("X-Requested-With", "XMLHttpRequest")
        .send_string(&format!("uname={uname}&password={pwd_enc}&fid=-1&t=true&refer=https%253A%252F%252Fi.chaoxing.com&forbidotherlogin=0&validate="))
}
