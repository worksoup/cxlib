use crate::{cookies::UserCookies, protocol};
use cxsign_dir::Dir;
use cxsign_login::LoginTrait;
use log::{info, trace};
use std::{
    hash::Hash,
    ops::{Deref, Index},
};
use ureq::Agent;

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
    /// 加载本地 Cookies 并返回 [`Session`].
    pub fn load_json(uname: &str) -> Result<Session, cxsign_error::Error> {
        let client = Agent::load_json(Dir::get_json_file_path(uname))?;
        let cookies = UserCookies::new(&client);
        let stu_name = Self::find_stu_name_in_html(&client)?;
        info!("用户[{}]加载 Cookies 成功！", stu_name);
        Ok(Session {
            agent: client,
            uname: uname.to_string(),
            stu_name,
            cookies,
        })
    }
    /// 类似于 [`Session::load_json`], 不过须传入加密后的密码以重新登录。重新登录后将 [`Session::store_json`] 以持久化 Cookies.
    pub fn relogin(uname: &str, enc_passwd: &str) -> Result<Session, cxsign_error::Error> {
        let client = Agent::login_enc(uname, enc_passwd)?;
        let cookies = UserCookies::new(&client);
        let stu_name = Self::find_stu_name_in_html(&client)?;
        info!("用户[{}]登录成功！", stu_name);
        let session = Session {
            agent: client,
            uname: uname.to_string(),
            stu_name,
            cookies,
        };
        session.store_json();
        Ok(session)
    }
    /// 先尝试 [`Session::load_json`], 如果发生错误且错误为登录过期或 Cookies 不存在，则 [`Session::relogin`]。
    pub fn load_json_or_relogin(
        uname: &str,
        enc_passwd: &str,
    ) -> Result<Session, cxsign_error::Error> {
        match Session::load_json(uname) {
            Ok(s) => Ok(s),
            Err(e) => match e {
                cxsign_error::Error::LoginExpired(_) => Session::relogin(uname, enc_passwd),
                cxsign_error::Error::IoError(e) => match e.kind() {
                    std::io::ErrorKind::NotFound => Session::relogin(uname, enc_passwd),
                    _ => Err(cxsign_error::Error::IoError(e)),
                },
                _ => Err(e),
            },
        }
    }
    /// 将 Cookies 保存在某位置。具体请查看代码：[`Session::store_json`].
    pub fn store_json(&self) {
        let store_path = Dir::get_json_file_path(self.get_uname());
        let mut writer = std::fs::File::create(store_path)
            .map(std::io::BufWriter::new)
            .unwrap();
        self.cookie_store().save_json(&mut writer).unwrap();
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
    fn find_stu_name_in_html(client: &Agent) -> Result<String, cxsign_error::Error> {
        let login_expired_err = || cxsign_error::Error::LoginExpired("无法获取姓名！".to_string());
        let r = protocol::account_manage(client)?;
        let html_content = r.into_string().unwrap();
        trace!("{html_content}");
        let e = html_content
            .find("colorBlue")
            .ok_or_else(login_expired_err)?;
        let html_content = html_content.index(e..html_content.len()).to_owned();
        let e = html_content.find('>').unwrap() + 1;
        let html_content = html_content.index(e..html_content.len()).to_owned();
        let name = html_content
            .index(0..html_content.find('<').unwrap())
            .trim();
        if name.is_empty() {
            return Err(cxsign_error::Error::LoginExpired("姓名为空！".to_string()));
        }
        Ok(name.to_owned())
    }
}

impl Deref for Session {
    type Target = Agent;
    fn deref(&self) -> &Agent {
        &self.agent
    }
}
