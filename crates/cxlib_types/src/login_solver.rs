use crate::error::LoginError;
use cx_enc_utils::crypto::pkcs7_pad;
use cxlib_error_utils::CxlibResultUtils;
use cxlib_protocol::{ProtocolItem, collect::UserProtocolTrait};
use log::{trace, warn};
use std::{
    any::{Any, TypeId},
    marker::PhantomData,
    ops::{Deref, Index},
};
use ureq::Agent;

pub trait LoginSolverTrait: Send + Sync + Any {
    type UserProtocol;
    fn login_type(&self) -> &str;
    fn is_logged_in(&self, agent: &Agent) -> bool
    where
        Self::UserProtocol: UserProtocolTrait;
    fn login_s(&self, account: &str, enc_passwd: &str) -> Result<Agent, LoginError>
    where
        Self::UserProtocol: UserProtocolTrait;
    fn pwd_enc(&self, pwd: String) -> Result<String, LoginError>;
}
impl<UserProtocol: 'static> LoginSolverTrait
    for Box<dyn LoginSolverTrait<UserProtocol = UserProtocol>>
{
    type UserProtocol = UserProtocol;

    fn login_type(&self) -> &str {
        self.deref().login_type()
    }

    fn is_logged_in(&self, agent: &Agent) -> bool
    where
        Self::UserProtocol: UserProtocolTrait,
    {
        self.deref().is_logged_in(agent)
    }

    fn login_s(&self, account: &str, enc_passwd: &str) -> Result<Agent, LoginError>
    where
        Self::UserProtocol: UserProtocolTrait,
    {
        self.deref().login_s(account, enc_passwd)
    }

    fn pwd_enc(&self, pwd: String) -> Result<String, LoginError> {
        self.deref().pwd_enc(pwd)
    }
}
pub struct DefaultLoginSolver<UserProtocol> {
    _t: PhantomData<UserProtocol>,
}
impl<T> Default for DefaultLoginSolver<T> {
    #[inline]
    fn default() -> Self {
        Self { _t: PhantomData }
    }
}
impl<UserProtocol> DefaultLoginSolver<UserProtocol>
where
    UserProtocol: UserProtocolTrait,
{
    pub fn find_stu_name_in_html(agent: &Agent) -> Result<String, LoginError> {
        let login_expired_err = || LoginError::LoginExpired("无法获取姓名！".to_string());
        let r = UserProtocol::account_manage(agent)?;
        let html_content = r.into_body().read_to_string().log_unwrap();
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
}
impl<UserProtocol> DefaultLoginSolver<UserProtocol> {
    pub fn des_enc(data: &[u8], key: [u8; 8]) -> String {
        use des::{
            Des,
            cipher::{BlockEncrypt as _, KeyInit as _, generic_array::GenericArray},
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
impl<UserProtocol> LoginSolverTrait for DefaultLoginSolver<UserProtocol>
where
    UserProtocol: Send + Sync + 'static,
{
    type UserProtocol = UserProtocol;

    #[inline]
    fn login_type(&self) -> &str {
        "default"
    }

    #[inline]
    fn is_logged_in(&self, agent: &Agent) -> bool
    where
        Self::UserProtocol: UserProtocolTrait,
    {
        Self::find_stu_name_in_html(agent).is_ok()
    }

    fn login_s(&self, account: &str, enc_passwd: &str) -> Result<Agent, LoginError>
    where
        Self::UserProtocol: UserProtocolTrait,
    {
        let client = Agent::new_with_config(
            Agent::config_builder()
                .user_agent(ProtocolItem::USER_AGENT)
                .build(),
        );
        let response = UserProtocol::login_enc(&client, account, enc_passwd)?;
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
        } = response
            .into_body()
            .read_json()
            .expect("json 反序列化失败！");
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

pub struct UntypedLoginSolver<UserProtocol>(Box<dyn LoginSolverTrait<UserProtocol = UserProtocol>>);
impl<UserProtocol: 'static> UntypedLoginSolver<UserProtocol> {
    pub fn from_typed<LoginSolver: LoginSolverTrait<UserProtocol = UserProtocol>>(
        t: LoginSolver,
    ) -> Self {
        UntypedLoginSolver(Box::new(t))
    }
    pub fn downcast<T: LoginSolverTrait>(self) -> Result<T, Self> {
        // Get `TypeId` of the type this function is instantiated with.
        let t = TypeId::of::<T>();

        // Get `TypeId` of the type in the trait object (`self`).
        let concrete = self.type_id();

        // Compare both `TypeId`s on equality.
        if t == concrete {
            let inner: Box<dyn Any> = self.0;
            // *unsafe { Ok(inner.downcast_unchecked::<T>()) }
            Ok(*inner.downcast().unwrap())
        } else {
            Err(self)
        }
    }
}

impl<UserProtocol> LoginSolverTrait for UntypedLoginSolver<UserProtocol>
where
    UserProtocol: 'static,
{
    type UserProtocol = UserProtocol;

    #[inline]
    fn login_type(&self) -> &str {
        self.0.login_type()
    }

    #[inline]
    fn is_logged_in(&self, agent: &ureq::Agent) -> bool
    where
        Self::UserProtocol: UserProtocolTrait,
    {
        self.0.is_logged_in(agent)
    }

    #[inline]
    fn login_s(&self, account: &str, enc_passwd: &str) -> Result<ureq::Agent, LoginError>
    where
        Self::UserProtocol: UserProtocolTrait,
    {
        self.0.login_s(account, enc_passwd)
    }

    #[inline]
    fn pwd_enc(&self, pwd: String) -> Result<String, LoginError> {
        self.0.pwd_enc(pwd)
    }
}

#[cfg(test)]
mod tests {}
