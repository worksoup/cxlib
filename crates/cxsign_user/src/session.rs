use crate::cookies::UserCookies;
use cxsign_dir::Dir;
use cxsign_login::{DefaultLoginSolver, LoginSolverTrait};
use cxsign_protocol::ProtocolItem;
use log::info;
use std::{hash::Hash, ops::Deref, path::Path};
use ureq::{Agent, AgentBuilder};

#[derive(Debug, Clone)]
pub struct Session {
    agent: Agent,
    uname: String,
    stu_name: String,
    cookies: UserCookies,
}

impl PartialEq for Session {
    fn eq(&self, other: &Self) -> bool {
        self.get_uid() == other.get_uid()
    }
}

impl Eq for Session {}

impl Hash for Session {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.get_uid().hash(state);
        self.get_fid().hash(state);
        self.get_stu_name().hash(state);
    }
}

impl Session {
    pub fn from_raw(
        uname: String,
        agent: Agent,
        cookies: UserCookies,
    ) -> Result<Session, cxsign_error::Error> {
        let stu_name = DefaultLoginSolver::find_stu_name_in_html(&agent)?;
        let session = Session {
            agent,
            uname: uname.to_string(),
            stu_name,
            cookies,
        };
        Ok(session)
    }
    pub fn load_cookies_raw<P: AsRef<Path>>(cookies_file: P) -> Result<Agent, std::io::Error> {
        let cookie_store = {
            let file = std::fs::File::open(cookies_file).map(std::io::BufReader::new)?;
            cookie_store::serde::json::load(file).unwrap()
        };
        Ok(AgentBuilder::new()
            .user_agent(&ProtocolItem::UserAgent.to_string())
            .cookie_store(cookie_store)
            .build())
    }
    /// 加载本地 Cookies 并返回 [`Session`].
    pub fn load_cookies(uid: &str, uname: &str) -> Result<Session, cxsign_error::Error> {
        let agent = Self::load_cookies_raw(Dir::get_json_file_path(uid))?;
        let cookies = UserCookies::new(&agent);
        let session = Self::from_raw(uname.to_string(), agent, cookies)?;
        info!("用户[{}]加载 Cookies 成功！", session.get_stu_name());
        Ok(session)
    }
    /// 类似于 [`Session::load_cookies`], 不过须传入加密后的密码以重新登录。重新登录后将 [`Session::store_cookies`] 以持久化 Cookies.
    pub fn relogin_raw<LoginSolver: LoginSolverTrait>(
        uname: &str,
        enc_pwd: &str,
        login_solver: &LoginSolver,
    ) -> Result<(Agent, UserCookies), cxsign_error::Error> {
        let agent = login_solver.login_enc(uname, enc_pwd)?;
        let cookies = UserCookies::new(&agent);
        Ok((agent, cookies))
    }
    /// 相当于 [`Session::relogin_raw`] 后 [`Session::from_raw`].
    pub fn relogin<LoginSolver: LoginSolverTrait>(
        uname: &str,
        enc_pwd: &str,
        login_solver: &LoginSolver,
    ) -> Result<Session, cxsign_error::Error> {
        let (agent, cookies) = Session::relogin_raw(uname, enc_pwd, login_solver)?;
        Self::store_cookies(&agent, cookies.get_uid())?;
        let session = Self::from_raw(uname.to_string(), agent, cookies)?;
        info!("用户[{}]登录成功！", session.get_stu_name());
        Ok(session)
    }
    /// 先尝试 [`Session::load_cookies`], 如果发生错误且错误为登录过期或 Cookies 不存在，则 [`Session::relogin`]。
    pub fn load_cookies_or_relogin<LoginSolver: LoginSolverTrait>(
        uname: &str,
        uid: &str,
        enc_passwd: &str,
        login_solver: &LoginSolver,
    ) -> Result<Session, cxsign_error::Error> {
        match Session::load_cookies(uid, uname) {
            Ok(s) => Ok(s),
            Err(e) => match e {
                cxsign_error::Error::LoginExpired(_) => {
                    Session::relogin(uname, enc_passwd, login_solver)
                }
                cxsign_error::Error::IoError(e) => match e.kind() {
                    std::io::ErrorKind::NotFound => {
                        Session::relogin(uname, enc_passwd, login_solver)
                    }
                    _ => Err(cxsign_error::Error::IoError(e)),
                },
                _ => Err(e),
            },
        }
    }
    /// 将 Cookies 保存在某位置。具体请查看代码：[`Session::store_cookies`].
    pub fn store_cookies(
        agent: &Agent,
        file_name_without_ext: &str,
    ) -> Result<(), cxsign_error::Error> {
        let store_path = Dir::get_json_file_path(file_name_without_ext);
        let mut writer = std::fs::File::create(store_path).map(std::io::BufWriter::new)?;
        cookie_store::serde::json::save(&agent.cookie_store(), &mut writer)
            .map_err(|e| cxsign_error::Error::LoginError(format!("Cookies 持久化失败：{e}")))
    }
    pub fn get_uid(&self) -> &str {
        self.cookies.get_uid()
    }
    pub fn get_fid(&self) -> &str {
        self.cookies.get_fid()
    }
    pub fn get_stu_name(&self) -> &str {
        &self.stu_name
    }
    pub fn get_uname(&self) -> &str {
        &self.uname
    }
    pub fn get_avatar_url(&self, size: usize) -> String {
        format!("https://photo.chaoxing.com/p/{}_{}", self.get_uid(), size)
    }
}

impl Deref for Session {
    type Target = Agent;
    fn deref(&self) -> &Agent {
        &self.agent
    }
}
