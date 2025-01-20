use crate::{cookies::UserCookies, Course};
use cxlib_error::{CourseError, CxlibResultUtils, LoginError};
use cxlib_login::{DefaultLoginSolver, LoginSolverTrait};
use cxlib_protocol::{collect::user as protocol, ProtocolItem};
use cxlib_store::Dir;
use log::info;
use serde::{Deserialize, Serialize};
use std::{hash::Hash, ops::Deref, path::Path};
use ureq::{serde_json, Agent, AgentBuilder};

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
    ) -> Result<Session, LoginError> {
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
    pub fn load_cookies(uid: &str, uname: &str) -> Result<Session, LoginError> {
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
    ) -> Result<(Agent, UserCookies), LoginError> {
        let agent = login_solver.login_s(uname, enc_pwd)?;
        let cookies = UserCookies::new(&agent);
        Ok((agent, cookies))
    }
    /// 相当于 [`Session::relogin_raw`] 后 [`Session::from_raw`].
    pub fn relogin<LoginSolver: LoginSolverTrait>(
        uname: &str,
        enc_pwd: &str,
        login_solver: &LoginSolver,
    ) -> Result<Session, LoginError> {
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
    ) -> Result<Session, LoginError> {
        match Session::load_cookies(uid, uname) {
            Ok(s) => Ok(s),
            Err(e) => match e {
                LoginError::LoginExpired(_) => Session::relogin(uname, enc_passwd, login_solver),
                LoginError::IoError(e) => match e.kind() {
                    std::io::ErrorKind::NotFound => {
                        Session::relogin(uname, enc_passwd, login_solver)
                    }
                    _ => Err(LoginError::IoError(e)),
                },
                _ => Err(e),
            },
        }
    }
    /// 将 Cookies 保存在某位置。具体请查看代码：[`Session::store_cookies`].
    pub fn store_cookies(agent: &Agent, file_name_without_ext: &str) -> Result<(), LoginError> {
        let store_path = Dir::get_json_file_path(file_name_without_ext);
        let mut writer = std::fs::File::create(store_path).map(std::io::BufWriter::new)?;
        cookie_store::serde::json::save(&agent.cookie_store(), &mut writer)
            .map_err(LoginError::CookiesStoreError)
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
    pub fn get_courses(&self) -> Result<Vec<Course>, CourseError> {
        let r = protocol::back_clazz_data(self.deref())?;
        let courses = Self::get_courses_from_response(r)?;
        info!("用户[{}]已获取课程列表。", self.get_stu_name());
        Ok(courses)
    }
    fn get_courses_from_response(r: ureq::Response) -> Result<Vec<Course>, CourseError> {
        #[derive(Deserialize, Serialize, Debug)]
        struct CourseRaw {
            id: i64,
            #[serde(rename = "teacherfactor")]
            teacher: String,
            #[serde(rename = "imageurl")]
            image_url: Option<String>,
            name: String,
        }
        #[derive(Deserialize, Serialize, Debug)]
        struct Courses {
            data: Vec<CourseRaw>,
        }

        #[derive(Deserialize, Serialize, Debug)]
        struct CourseContent {
            course: Option<Courses>,
        }

        #[derive(Deserialize, Serialize, Debug)]
        struct ClassRaw {
            #[serde(rename = "key")]
            id: serde_json::Value,
            content: CourseContent,
        }

        #[derive(Deserialize, Serialize, Debug)]
        struct GetCoursesR {
            #[serde(rename = "channelList")]
            channel_list: Option<Vec<ClassRaw>>,
        }
        let r: GetCoursesR = r.into_json().log_unwrap();
        let mut arr = Vec::new();
        if let Some(channel_list) = r.channel_list {
            for c in channel_list {
                if let Some(data) = c.content.course {
                    for course in data.data {
                        if c.id.is_i64() {
                            arr.push(Course::new(
                                course.id,
                                c.id.as_i64().unwrap(),
                                course.teacher.as_str(),
                                course.image_url.unwrap_or("".into()).as_str(),
                                course.name.as_str(),
                            ))
                        }
                    }
                }
            }
            Ok(arr)
        } else {
            Err(LoginError::LoginExpired(
                "`channelList` 字段为空!".to_string(),
            ))?
        }
    }
}

impl Deref for Session {
    type Target = Agent;
    fn deref(&self) -> &Agent {
        &self.agent
    }
}
