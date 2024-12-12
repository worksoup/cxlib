use cxlib_error::{LoginError, CxlibResultUtils};
use cxlib_protocol::{collect::user as protocol, ProtocolItem};
use cxlib_utils::pkcs7_pad;
use log::{trace, warn};
use onceinit::{OnceInit, OnceInitState, StaticDefault};
use std::{
    collections::HashMap,
    ops::Index,
    sync::{Arc, RwLock},
};
use ureq::{serde, Agent, AgentBuilder};

pub trait LoginSolverTrait: Send + Sync {
    fn login_type(&self) -> &str;
    fn is_logged_in(&self, agent: &Agent) -> bool;
    fn login_s(&self, account: &str, enc_passwd: &str) -> Result<Agent, LoginError>;
    fn pwd_enc(&self, pwd: String) -> Result<String, LoginError>;
}
pub struct DefaultLoginSolver;
impl DefaultLoginSolver {
    pub fn find_stu_name_in_html(agent: &Agent) -> Result<String, LoginError> {
        let login_expired_err = || LoginError::LoginExpired("无法获取姓名！".to_string());
        let r = protocol::account_manage(agent)?;
        let html_content = r.into_string().log_unwrap();
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
            return Err(LoginError::LoginExpired("姓名为空！".to_string()));
        }
        Ok(name.to_owned())
    }
    pub fn des_enc(data: &[u8], key: [u8; 8]) -> String {
        use des::{
            cipher::{generic_array::GenericArray, BlockEncrypt as _, KeyInit as _},
            Des,
        };
        let key = GenericArray::from(key);
        let des = Des::new(&key);
        let mut data_block_enc = Vec::new();
        for block in pkcs7_pad(data) {
            let mut block = GenericArray::from(block);
            des.encrypt_block(&mut block);
            let mut block = block.to_vec();
            data_block_enc.append(&mut block);
        }
        hex::encode(data_block_enc)
    }
}
impl LoginSolverTrait for DefaultLoginSolver {
    fn login_type(&self) -> &str {
        "default"
    }

    fn is_logged_in(&self, agent: &Agent) -> bool {
        Self::find_stu_name_in_html(agent).is_ok()
    }

    fn login_s(&self, account: &str, enc_passwd: &str) -> Result<Agent, LoginError> {
        let cookie_store = cookie_store::CookieStore::new(None);
        let client = AgentBuilder::new()
            .user_agent(&ProtocolItem::UserAgent.to_string())
            .cookie_store(cookie_store)
            .build();
        let response = protocol::login_enc(&client, account, enc_passwd)?;
        /// TODO: 存疑
        #[derive(serde::Deserialize)]
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
        } = response.into_json().expect("json 反序列化失败！");
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
            for mes in &mes {
                warn!("{mes:?}");
            }
            return Err(LoginError::ServerError(format!("{mes:?}")));
        }
        Ok(client)
    }

    fn pwd_enc(&self, pwd: String) -> Result<String, LoginError> {
        let pwd = pwd.as_bytes();
        if (8..=16).contains(&pwd.len()) {
            Ok(Self::des_enc(pwd, b"u2oh6Vu^".to_owned()))
        } else {
            Err(LoginError::CryptoError("密码长度不规范".to_string()))
        }
    }
}

/// LoginSolver 全局列表，若要支持新的登录方式，请实现 [`LoginSolverTrait`], 并进行[注册](LoginSolvers::register)。
pub struct LoginSolvers(Arc<RwLock<HashMap<String, Box<dyn LoginSolverTrait>>>>);
impl LoginSolvers {
    /// 注册登录协议，参数须实现 [`LoginSolverTrait`].
    pub fn register(solver: impl LoginSolverTrait + 'static) -> Result<(), LoginError> {
        let solver = Box::new(solver);
        LOGIN_SOLVERS
            .0
            .write()
            .unwrap()
            .insert(solver.login_type().to_string(), solver);
        Ok(())
    }
}
static LOGIN_SOLVERS: OnceInit<LoginSolvers> = OnceInit::new();
unsafe impl StaticDefault for LoginSolvers {
    fn static_default() -> &'static Self {
        if let OnceInitState::UNINITIALIZED = LOGIN_SOLVERS.get_state() {
            let mut map = HashMap::new();
            let solver: Box<dyn LoginSolverTrait> = Box::new(DefaultLoginSolver);
            map.insert(solver.login_type().to_owned(), solver);
            let login_solvers = LoginSolvers(Arc::new(RwLock::new(map)));
            LOGIN_SOLVERS
                .set_boxed_data(Box::new(login_solvers))
                .unwrap();
        }
        LOGIN_SOLVERS.as_data()
    }
}
/// # [`LoginSolverWrapper`]
/// [`LoginSolverTrait`] 的包装，需要从字符串构造 LoginSolver 时请使用该类型。
/// ``` rust
/// use cxlib_user::LoginSolverWrapper;
/// let solver = LoginSolverWrapper::new("login_type");
/// ```
pub struct LoginSolverWrapper<'s>(&'s str);

impl LoginSolverTrait for LoginSolverWrapper<'_> {
    fn login_type(&self) -> &str {
        self.0
    }

    fn is_logged_in(&self, agent: &Agent) -> bool {
        LOGIN_SOLVERS
            .0
            .read()
            .unwrap()
            .get(self.0)
            .is_some_and(|l| l.is_logged_in(agent))
    }

    fn login_s(&self, account: &str, enc_passwd: &str) -> Result<Agent, LoginError> {
        LOGIN_SOLVERS
            .0
            .read()
            .unwrap()
            .get(self.0)
            .ok_or_else(|| LoginError::UnsupportedProtocol)?
            .login_s(account, enc_passwd)
    }

    fn pwd_enc(&self, pwd: String) -> Result<String, LoginError> {
        LOGIN_SOLVERS
            .0
            .read()
            .unwrap()
            .get(self.0)
            .ok_or_else(|| LoginError::UnsupportedProtocol)?
            .pwd_enc(pwd)
    }
}
impl LoginSolverWrapper<'_> {
    pub fn new(login_type: &str) -> LoginSolverWrapper {
        LoginSolverWrapper(login_type)
    }
}
#[cfg(test)]
mod tests {}
