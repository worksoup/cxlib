use crate::{cookies::UserCookies, Class, ClassId, ClassInfo, Course, RawCourse};
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
        self.uid() == other.uid()
    }
}

impl Eq for Session {}

impl Hash for Session {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uid().hash(state);
        self.fid().hash(state);
        self.name().hash(state);
    }
}

impl Session {
    #[inline]
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
    #[inline]
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
    #[inline]
    pub fn load_cookies(uid: &str, uname: &str) -> Result<Session, LoginError> {
        let agent = Self::load_cookies_raw(Dir::get_json_file_path(uid))?;
        let cookies = UserCookies::new(&agent);
        let session = Self::from_raw(uname.to_string(), agent, cookies)?;
        info!("用户[{}]加载 Cookies 成功！", session.name());
        Ok(session)
    }
    /// 类似于 [`Session::load_cookies`], 不过须传入加密后的密码以重新登录。重新登录后将 [`Session::store_cookies`] 以持久化 Cookies.
    #[inline]
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
    #[inline]
    pub fn relogin<LoginSolver: LoginSolverTrait>(
        uname: &str,
        enc_pwd: &str,
        login_solver: &LoginSolver,
    ) -> Result<Session, LoginError> {
        let (agent, cookies) = Session::relogin_raw(uname, enc_pwd, login_solver)?;
        Self::store_cookies(&agent, cookies.uid())?;
        let session = Self::from_raw(uname.to_string(), agent, cookies)?;
        info!("用户[{}]登录成功！", session.name());
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
    #[inline]
    pub fn store_cookies(agent: &Agent, file_name_without_ext: &str) -> Result<(), LoginError> {
        let store_path = Dir::get_json_file_path(file_name_without_ext);
        let mut writer = std::fs::File::create(store_path).map(std::io::BufWriter::new)?;
        cookie_store::serde::json::save(&agent.cookie_store(), &mut writer)
            .map_err(LoginError::CookiesStoreError)
    }
    #[inline]
    pub fn uid(&self) -> &str {
        self.cookies.uid()
    }
    #[inline]
    pub fn fid(&self) -> &str {
        self.cookies.fid()
    }
    #[inline]
    pub fn name(&self) -> &str {
        &self.stu_name
    }
    #[inline]
    pub fn uname(&self) -> &str {
        &self.uname
    }
    #[inline]
    pub fn avatar_url(&self, size: usize) -> String {
        format!("https://photo.chaoxing.com/p/{}_{}", self.uid(), size)
    }
}
impl Session {
    #[inline]
    pub fn get_courses(&self) -> Result<Vec<Course>, CourseError> {
        let classes = self.get_classes()?;
        let mut courses = Vec::new();

        for class in classes {
            courses.append(&mut class.into_courses());
        }
        info!("用户[{}]已获取课程列表。", self.name());
        Ok(courses)
    }
    pub fn get_classes(&self) -> Result<Vec<Class>, CourseError> {
        let r = protocol::back_clazz_data(self.deref())?;
        #[derive(Deserialize, Serialize, Debug, Default)]
        struct Courses {
            data: Vec<RawCourse>,
        }

        #[derive(Deserialize, Serialize, Debug)]
        struct CourseContent {
            course: Option<Courses>,
            // TODO: 需要使用该字段。
            // 0 | 1, 代表班级开课或结束。
            state: u8,
        }

        #[derive(Deserialize, Serialize, Debug)]
        struct ClassRaw {
            #[serde(rename = "key")]
            id: serde_json::Value,
            content: CourseContent,
        }
        impl ClassRaw {
            pub fn into_class(self) -> Class {
                let Self { id, content } = self;
                let id = if id.is_i64() {
                    ClassId::Id(id.as_i64().unwrap())
                } else if id.is_string() {
                    ClassId::TeacherId(
                        id.as_str()
                            .unwrap()
                            .strip_prefix("tea_")
                            .expect(
                                "ClassId 格式不正确（不以 `tea_` 开头），请检查 API 是否存在更新。",
                            )
                            .parse()
                            .unwrap(),
                    )
                } else {
                    panic!("JSON 解析失败：存在意外类型的 ClassId, 请检查 API 是否存在更新。")
                };
                let CourseContent { course, state } = content;
                let courses = course.unwrap_or_default().data;
                let ended = state == 0;
                Class::new(courses, ClassInfo::new(id, ended))
            }
        }

        #[derive(Deserialize, Serialize, Debug)]
        struct GetCoursesR {
            #[serde(rename = "channelList")]
            channel_list: Option<Vec<ClassRaw>>,
        }
        let r2: GetCoursesR = r.into_json().log_unwrap();
        let classes = if let Some(channel_list) = r2.channel_list {
            channel_list.into_iter().map(|c| c.into_class()).collect()
        } else {
            Err(LoginError::LoginExpired(
                "`channelList` 字段为空!".to_string(),
            ))?
        };
        info!("用户[{}]已获取班级列表。", self.name());
        Ok(classes)
    }
}

impl Deref for Session {
    type Target = Agent;
    #[inline]
    fn deref(&self) -> &Agent {
        &self.agent
    }
}
