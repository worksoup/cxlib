use crate::{
    Class, ClassInfo, CourseWithInfo, DefaultLoginSolver, LoginSolverTrait, RawCourse,
    UntypedLoginSolver,
    cookies::UserCookies,
    error::{CourseError, LoginError},
};
use cxlib_error::AgentError;
use cxlib_error_utils::CxlibResultUtils;
use cxlib_protocol::{ProtocolItem, collect::UserProtocolTrait};
use log::info;
use serde::Deserialize;
use std::{hash::Hash, marker::PhantomData, ops::Deref};
use ureq::Agent;

#[derive(Debug)]
pub struct Session<UserProtocol> {
    agent: Agent,
    uname: String,
    stu_name: String,
    cookies: UserCookies,
    _p: PhantomData<UserProtocol>,
}
impl<U> Session<U> {
    pub fn into<Other>(self) -> Session<Other> {
        let Session {
            agent,
            uname,
            stu_name,
            cookies,
            _p,
        } = self;
        Session {
            agent,
            uname,
            stu_name,
            cookies,
            _p: Default::default(),
        }
    }
}
impl<T> Clone for Session<T> {
    #[inline]
    fn clone(&self) -> Session<T> {
        Session {
            agent: Clone::clone(&self.agent),
            uname: Clone::clone(&self.uname),
            stu_name: Clone::clone(&self.stu_name),
            cookies: Clone::clone(&self.cookies),
            _p: Clone::clone(&self._p),
        }
    }
}

impl<U> PartialEq for Session<U> {
    fn eq(&self, other: &Self) -> bool {
        self.uid() == other.uid()
    }
}

impl<U> Eq for Session<U> {}

impl<U> Hash for Session<U> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uid().hash(state);
        self.fid().hash(state);
        self.name().hash(state);
    }
}

impl<UserProtocol> Session<UserProtocol>
where
    UserProtocol: UserProtocolTrait,
{
    #[inline]
    pub fn from_raw(
        uname: String,
        stu_name: String,
        agent: Agent,
        cookies: UserCookies,
    ) -> Result<Self, LoginError> {
        let session = Self {
            agent,
            uname,
            stu_name,
            cookies,
            _p: Default::default(),
        };
        Ok(session)
    }
    /// 加载本地 Cookies 并返回 [`Session`].
    #[inline]
    pub fn load_cookies<R: std::io::BufRead>(
        uname: String,
        stu_name: String,
        r: R,
    ) -> Result<Self, LoginError> {
        let agent = Self::load_cookies_raw(r)?;
        let cookies = UserCookies::new(&agent);
        let session = Self::from_raw(uname, stu_name, agent, cookies)?;
        info!("用户[{}]加载 Cookies 成功！", session.name());
        Ok(session)
    }
}
impl<UserProtocol> Session<UserProtocol>
where
    UserProtocol: UserProtocolTrait + 'static,
{
    /// 先尝试 [`Session::load_cookies`], 如果发生错误且错误为登录过期或 Cookies 不存在，则 [`Session::relogin`]。
    pub fn load_cookies_or_relogin<R: std::io::BufRead, W: std::io::Write>(
        enc_passwd: &str,
        uname: &str,
        stu_name: String,
        r: R,
        w: &mut W,
        login_solver: &UntypedLoginSolver<UserProtocol>,
    ) -> Result<Self, LoginError> {
        match Self::load_cookies(uname.to_owned(), stu_name, r) {
            Ok(s) => Ok(s),
            Err(e) => match e {
                LoginError::LoginExpired(_) => Self::relogin(uname, enc_passwd, w, login_solver),
                LoginError::IoError(e) => match e.kind() {
                    std::io::ErrorKind::NotFound => {
                        Self::relogin(uname, enc_passwd, w, login_solver)
                    }
                    _ => Err(LoginError::IoError(e)),
                },
                _ => Err(e),
            },
        }
    }
    /// 相当于 [`Session::relogin_raw`] 后 [`Session::from_raw`].
    #[inline]
    pub fn relogin<W: std::io::Write>(
        uname: &str,
        enc_pwd: &str,
        writer: &mut W,
        login_solver: &UntypedLoginSolver<UserProtocol>,
    ) -> Result<Self, LoginError> {
        let (agent, cookies) = Self::relogin_raw(uname, enc_pwd, login_solver)?;
        Self::store_cookies(&agent, writer)?;
        let stu_name = DefaultLoginSolver::<UserProtocol>::find_stu_name_in_html(&agent)?;
        let session = Self::from_raw(uname.to_string(), stu_name, agent, cookies)?;
        info!("用户[{}]登录成功！", session.name());
        Ok(session)
    }
}
impl<U> Session<U> {
    #[inline]
    pub fn load_cookies_raw<R: std::io::BufRead>(r: R) -> Result<Agent, std::io::Error> {
        let config = Agent::config_builder()
            .user_agent(ProtocolItem::USER_AGENT)
            .build();
        let agent = Agent::new_with_config(config);
        agent.cookie_jar_lock().load_json(r).log_unwrap();
        Ok(agent)
    }
    /// 将 Cookies 保存在某位置。具体请查看代码：[`Session::store_cookies`].
    #[inline]
    pub fn store_cookies<W: std::io::Write>(
        agent: &Agent,
        writer: &mut W,
    ) -> Result<(), LoginError> {
        agent
            .cookie_jar_lock()
            .save_json(writer)
            .map_err(|e| LoginError::CookiesStoreError(AgentError::from(e)))
    }
}
impl<UserProtocol> Session<UserProtocol>
where
    UserProtocol: UserProtocolTrait + 'static,
{
    /// 类似于 [`Session::load_cookies`], 不过须传入加密后的密码以重新登录。重新登录后将 [`Session::store_cookies`] 以持久化 Cookies.
    #[inline]
    pub fn relogin_raw(
        uname: &str,
        enc_pwd: &str,
        login_solver: &UntypedLoginSolver<UserProtocol>,
    ) -> Result<(Agent, UserCookies), LoginError> {
        let agent = login_solver.login_s(uname, enc_pwd)?;
        let cookies = UserCookies::new(&agent);
        Ok((agent, cookies))
    }
}

impl<UserProtocol> Session<UserProtocol> {
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
impl<UserProtocol: UserProtocolTrait> Session<UserProtocol> {
    #[inline]
    pub fn get_courses(&self) -> Result<Vec<CourseWithInfo>, CourseError> {
        let classes = self.get_classes()?;
        let mut courses = Vec::new();

        for class in classes {
            courses.append(&mut class.into_courses());
        }
        info!("用户[{}]已获取课程列表。", self.name());
        Ok(courses)
    }
    pub fn get_classes(&self) -> Result<Vec<Class>, CourseError> {
        let r = UserProtocol::back_clazz_data(self.deref())?;
        #[derive(Deserialize, Debug, Default)]
        struct Courses {
            data: Vec<RawCourse>,
        }

        #[derive(Deserialize, Debug)]
        struct RawClass {
            #[serde(rename = "clazzId")]
            id: i64,
            // 0 | 1, 代表班级开课或结束。
            state: Option<u8>,
        }
        #[derive(Deserialize, Debug)]
        struct CourseContent {
            // TODO: 该字段不应为 Option.
            // TODO: 深层原因为，Class 与 Course 并非一对多的关系。
            // TODO: 而是多对多的关系。
            // TODO: 同时，当前实现混淆了两者的 ID, 可能会出现一些错误。
            course: Option<Courses>,
            #[serde(rename = "clazz")]
            classes: Option<Vec<RawClass>>,
            // TODO: 需要测试 `Option` 是否可以为 `flatten`.
            #[serde(flatten)]
            raw_course: Option<RawCourse>,
            // 0 | 1, 代表班级开课或结束。
            state: Option<u8>,
        }

        #[derive(Deserialize, Debug)]
        struct ClassRaw {
            // 可能为 ClassId, 也可能是 CourseId.
            #[serde(rename = "key")]
            id: serde_json::Value,
            content: CourseContent,
        }
        impl ClassRaw {
            pub fn into_class(self) -> Vec<Class> {
                let Self { id, content } = self;
                if id.is_i64() {
                    let class_id = id.as_i64().unwrap();
                    let CourseContent { course, state, .. } = content;
                    let courses = course.unwrap_or_default().data;
                    vec![Class::new(
                        courses,
                        ClassInfo::new_with_state(class_id, state),
                    )]
                } else if id.is_string() {
                    // let course_id = id.as_str()
                    //     .unwrap()
                    //     .strip_prefix("tea_")
                    //     .expect(
                    //         "CourseId 格式不正确（不以 `tea_` 开头），请检查 API 是否存在更新。",
                    //     )
                    //     .parse()
                    //     .unwrap();
                    let CourseContent {
                        classes,
                        raw_course,
                        ..
                    } = content;
                    classes
                        .map(|classes| {
                            classes.into_iter().map(|clazz| {
                                Class::new(
                                    raw_course.clone().into_iter().collect(),
                                    ClassInfo::new_with_state(clazz.id, clazz.state),
                                )
                            })
                        })
                        .into_iter()
                        .flatten()
                        .collect::<Vec<_>>()
                } else {
                    panic!(
                        "JSON 解析失败：存在意外类型的 ClassId 或 CourseId(json 键为 \"key\"), 请检查 API 是否存在更新。"
                    )
                }
            }
        }

        #[derive(Deserialize, Debug)]
        struct GetCoursesR {
            #[serde(rename = "channelList")]
            channel_list: Option<Vec<ClassRaw>>,
        }
        let r2: GetCoursesR = r.into_body().read_json().log_unwrap();
        let classes = if let Some(channel_list) = r2.channel_list {
            channel_list
                .into_iter()
                .flat_map(|c| c.into_class())
                .collect()
        } else {
            Err(LoginError::LoginExpired(
                "`channelList` 字段为空!".to_string(),
            ))?
        };
        info!("用户[{}]已获取班级列表。", self.name());
        Ok(classes)
    }
}

impl<T> Deref for Session<T> {
    type Target = Agent;
    #[inline]
    fn deref(&self) -> &Agent {
        &self.agent
    }
}
