use crate::{cookies::UserCookies, protocol};
use cxsign_dir::Dir;
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
    pub fn load_json(uname: &str) -> Result<Self, Box<ureq::Error>> {
        let client = cxsign_login::load_json(Dir::get_json_file_path(uname));
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
    pub fn relogin(uname: &str, enc_passwd: &str) -> Result<Session, cxsign_error::Error> {
        let client = cxsign_login::login_enc(uname, enc_passwd)?;
        let cookies = UserCookies::new(&client);
        let stu_name = Self::find_stu_name_in_html(&client)?;
        info!("用户[{}]登录成功！", stu_name);
        let session = Session {
            agent: client,
            uname: uname.to_string(),
            stu_name,
            cookies,
        };
        Ok(session)
    }
    pub fn login(uname: &str, enc_passwd: &str) -> Result<Session, cxsign_error::Error> {
        let session = Session::relogin(uname, enc_passwd)?;
        session.store_json();
        Ok(session)
    }
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
    fn find_stu_name_in_html(client: &Agent) -> Result<String, Box<ureq::Error>> {
        let r = protocol::account_manage(client)?;
        let html_content = r.into_string().unwrap();
        trace!("{html_content}");
        let e = html_content.find("colorBlue").unwrap();
        let html_content = html_content.index(e..html_content.len()).to_owned();
        let e = html_content.find('>').unwrap() + 1;
        let html_content = html_content.index(e..html_content.len()).to_owned();
        let name = html_content
            .index(0..html_content.find('<').unwrap())
            .trim();
        Ok(name.to_owned())
    }
}

impl Deref for Session {
    type Target = Agent;
    fn deref(&self) -> &Agent {
        &self.agent
    }
}
