use crate::{
    Class, CourseWithInfo, DefaultLoginSolver, LoginSolverTrait, UntypedLoginSolver,
    cookies::UserCookies,
    error::{CourseError, LoginError},
};
use cxlib_error::AgentError;
use cxlib_error_utils::CxlibResultUtils;
use cxlib_protocol::{ProtocolItem, collect::UserProtocolTrait};
use getset2::Getset2;
use log::info;
use std::{hash::Hash, ops::Deref};
use ureq::Agent;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Getset2)]
#[getset2(get_ref(pub))]
pub struct SessionUserInfo {
    uname: String,
    name: String,
    cookies: UserCookies,
}
impl SessionUserInfo {
    #[inline]
    pub fn new(uname: String, name: String, cookies: UserCookies) -> Self {
        Self {
            uname,
            name,
            cookies,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Session {
    user_info: SessionUserInfo,
    agent: Agent,
}
impl Session {
    #[inline]
    pub fn from_raw(user_info: SessionUserInfo, agent: Agent) -> Result<Self, LoginError> {
        let session = Self { agent, user_info };
        Ok(session)
    }
}
impl Session {
    #[inline]
    pub fn user_info(&self) -> &SessionUserInfo {
        &self.user_info
    }
    #[inline]
    pub fn uid(&self) -> &String {
        self.user_info().cookies().uid()
    }
    #[inline]
    pub fn fid(&self) -> &String {
        self.user_info().cookies().fid()
    }
    #[inline]
    pub fn name(&self) -> &String {
        self.user_info().name()
    }
    #[inline]
    pub fn uname(&self) -> &String {
        self.user_info().uname()
    }
    #[inline]
    pub fn avatar_url(&self, size: usize) -> String {
        format!("https://photo.chaoxing.com/p/{}_{}", self.uid(), size)
    }
}
impl Session {
    /// 加载本地 Cookies 并返回 [`Session`].
    #[inline]
    pub fn load_cookies<R: std::io::BufRead>(
        uname: String,
        stu_name: String,
        r: R,
    ) -> Result<Self, LoginError> {
        let agent = Self::load_cookies_raw(r)?;
        let cookies = UserCookies::new(&agent);
        let session = Self::from_raw(SessionUserInfo::new(uname, stu_name, cookies), agent)?;
        info!("用户[{}]加载 Cookies 成功！", session.name());
        Ok(session)
    }
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
impl Session {
    #[inline]
    pub fn get_courses<UserProtocol>(&self) -> Result<Vec<CourseWithInfo>, CourseError>
    where
        UserProtocol: UserProtocolTrait,
    {
        let classes = self.get_classes::<UserProtocol>()?;
        let mut courses = Vec::new();

        for class in classes {
            courses.append(&mut class.into_courses());
        }
        info!("用户[{}]已获取课程列表。", self.name());
        Ok(courses)
    }
    pub fn get_classes<UserProtocol>(&self) -> Result<Vec<Class>, CourseError>
    where
        UserProtocol: UserProtocolTrait,
    {
        let raw_classes = UserProtocol::get_classes(self.deref()).map_err(|e| match e {
            cxlib_protocol::ProtocolError::AgentError(agent_error) => agent_error.into(),
            cxlib_protocol::ProtocolError::DataParseError(e) => LoginError::LoginExpired(e),
            _ => unreachable!("`UserProtocol::back_clazz_data` 不会返回该类型的错误。"),
        })?;
        let classes = raw_classes
            .into_iter()
            .flat_map(|raw_clazz| raw_clazz.into_class())
            .collect();
        info!("用户[{}]已获取班级列表。", self.name());
        Ok(classes)
    }
}
impl Session {
    /// 先尝试 [`Session::load_cookies`], 如果发生错误且错误为登录过期或 Cookies 不存在，则 [`Session::relogin`]。
    pub fn load_cookies_or_relogin<R: std::io::BufRead, W: std::io::Write, UserProtocol>(
        enc_passwd: &str,
        uname: &str,
        stu_name: String,
        r: R,
        w: &mut W,
        login_solver: &UntypedLoginSolver<UserProtocol>,
    ) -> Result<Self, LoginError>
    where
        UserProtocol: UserProtocolTrait + 'static,
    {
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
    pub fn relogin<W, UserProtocol>(
        uname: &str,
        enc_pwd: &str,
        writer: &mut W,
        login_solver: &UntypedLoginSolver<UserProtocol>,
    ) -> Result<Self, LoginError>
    where
        W: std::io::Write,
        UserProtocol: UserProtocolTrait + 'static,
    {
        let (agent, cookies) = Self::relogin_raw(uname, enc_pwd, login_solver)?;
        Self::store_cookies(&agent, writer)?;
        let stu_name = DefaultLoginSolver::<UserProtocol>::find_stu_name_in_html(&agent)?;
        let session = Self::from_raw(
            SessionUserInfo::new(uname.to_string(), stu_name, cookies),
            agent,
        )?;
        info!("用户[{}]登录成功！", session.name());
        Ok(session)
    }
    /// 类似于 [`Session::load_cookies`], 不过须传入加密后的密码以重新登录。重新登录后将 [`Session::store_cookies`] 以持久化 Cookies.
    #[inline]
    pub fn relogin_raw<UserProtocol>(
        uname: &str,
        enc_pwd: &str,
        login_solver: &UntypedLoginSolver<UserProtocol>,
    ) -> Result<(Agent, UserCookies), LoginError>
    where
        UserProtocol: UserProtocolTrait + 'static,
    {
        let agent = login_solver.login_s(uname, enc_pwd)?;
        let cookies = UserCookies::new(&agent);
        Ok((agent, cookies))
    }
}
impl Deref for Session {
    type Target = Agent;
    #[inline]
    fn deref(&self) -> &Agent {
        &self.agent
    }
}
