use super::cookies::UserCookies;
use crate::activity::{
    sign::{Sign, SignTrait},
    Activity, OtherActivity,
};
use crate::course::Course;
use crate::protocol;
use crate::protocol::UA;
use crate::utils::CONFIG_DIR;
use serde::Deserialize;
use std::fs::File;
use std::{
    hash::Hash,
    ops::{Deref, Index},
};
use ureq::{Agent, AgentBuilder};

#[derive(Debug)]
pub struct Session {
    agent: Agent,
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
    pub async fn load_json<P: AsRef<std::path::Path>>(
        cookies_file: P,
    ) -> Result<Self, ureq::Error> {
        let cookie_store = {
            let file = std::fs::File::open(cookies_file)
                .map(std::io::BufReader::new)
                .unwrap();
            cookie_store::CookieStore::load_json(file).unwrap()
        };
        let cookies = {
            let mut cookies = Vec::new();
            for c in cookie_store.iter_any() {
                cookies.push(c.to_owned())
            }
            cookies
        };
        let cookies = UserCookies::new(cookies);
        let client = AgentBuilder::new()
            .user_agent(UA)
            .cookie_store(cookie_store)
            .build();
        let stu_name = Self::find_stu_name_in_html(&client).await?;
        println!("用户[{}]加载 Cookies 成功！", stu_name);
        Ok(Session {
            agent: client,
            stu_name,
            cookies,
        })
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

    pub async fn login(account: &str, enc_passwd: &str) -> Result<Session, ureq::Error> {
        let cookie_store = cookie_store::CookieStore::new(None);
        let client = AgentBuilder::new()
            .user_agent(UA)
            .cookie_store(cookie_store)
            .build();
        let response = login::protocol::login_enc(&client, account, enc_passwd).await?;
        /// TODO: 存疑
        #[derive(Deserialize)]
        struct LoginR {
            url: Option<String>,
            msg1: Option<String>,
            msg2: Option<String>,
            status: bool,
        }
        let LoginR {
            status,
            url,
            msg1,
            msg2,
        } = response.into_json().unwrap();
        let mut mes = Vec::new();
        if let Some(url) = url {
            mes.push(url);
        }
        if let Some(msg1) = msg1 {
            mes.push(msg1);
        }
        if let Some(msg2) = msg2 {
            mes.push(msg2);
        }
        if !status {
            for mes in mes {
                eprintln!("{mes:?}");
            }
            panic!("登录失败！");
        }
        // Write store back to disk
        let mut writer = std::fs::File::create(CONFIG_DIR.join(account.to_string() + ".json"))
            .map(std::io::BufWriter::new)
            .unwrap();
        let store = {
            let mut r = Vec::new();
            for s in client.cookie_store().iter_any() {
                r.push(s.to_owned());
            }
            r
        };
        client.cookie_store().save_json(&mut writer).unwrap();
        let cookies = UserCookies::new(store);
        let stu_name = Self::find_stu_name_in_html(&client).await?;
        println!("用户[{}]登录成功！", stu_name);
        Ok(Session {
            agent: client,
            stu_name,
            cookies,
        })
    }

    pub async fn get_courses(&self) -> Result<Vec<Course>, ureq::Error> {
        let r = protocol::back_clazz_data(self.deref()).await?;
        let courses = Course::get_list_from_response(r)?;
        println!("用户[{}]已获取课程列表。", self.stu_name);
        Ok(courses)
    }
    async fn find_stu_name_in_html(client: &Agent) -> Result<String, ureq::Error> {
        let r = protocol::account_manage(client)?;
        let html_content = r.into_string().unwrap();
        #[cfg(debug_assertions)]
        println!("{html_content}");
        let e = html_content.find("colorBlue").unwrap();
        let html_content = html_content.index(e..html_content.len()).to_owned();
        let e = html_content.find('>').unwrap() + 1;
        let html_content = html_content.index(e..html_content.len()).to_owned();
        let name = html_content
            .index(0..html_content.find('<').unwrap())
            .trim();
        Ok(name.to_owned())
    }
    pub async fn get_pan_token(&self) -> Result<String, ureq::Error> {
        let r = protocol::pan_token(self).await?;
        #[derive(Deserialize)]
        struct Tmp {
            #[serde(alias = "_token")]
            token: String,
        }
        let r: Tmp = r.into_json().unwrap();
        Ok(r.token)
    }

    pub async fn upload_image(&self, file: &File, file_name: &str) -> Result<String, ureq::Error> {
        let token = self.get_pan_token().await?;
        let r = protocol::pan_upload(self, file, self.get_uid(), &token, file_name).await?;
        #[derive(Deserialize)]
        struct Tmp {
            #[serde(alias = "objectId")]
            object_id: String,
        }
        let tmp: Tmp = r.into_json().unwrap();
        Ok(tmp.object_id)
    }
}

impl Session {
    pub async fn get_all_activities(
        &self,
    ) -> Result<(Vec<Sign>, Vec<Sign>, Vec<OtherActivity>), ureq::Error> {
        let mut 有效签到列表 = Vec::new();
        let mut 其他签到列表 = Vec::new();
        let mut 非签到活动列表 = Vec::new();
        let 课程列表 = self.get_courses().await?;
        for c in 课程列表 {
            let item = Activity::get_list_from_course(self, &c)?;
            for a in item {
                if let Activity::签到(签到) = a {
                    if 签到.is_valid() {
                        有效签到列表.push(签到);
                    } else {
                        其他签到列表.push(签到);
                    }
                } else if let Activity::非签到活动(非签到活动) = a {
                    非签到活动列表.push(非签到活动);
                }
            }
        }
        有效签到列表.sort();
        Ok((有效签到列表, 其他签到列表, 非签到活动列表))
    }
}

impl Deref for Session {
    type Target = Agent;
    fn deref(&self) -> &Agent {
        &self.agent
    }
}
