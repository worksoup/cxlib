use super::internal_data::*;
use crate::StoreError;
use bincode::{Decode, Encode};
use cxlib_internal::types::LoginSolverTrait;
use std::{borrow::Borrow, fmt::Display, path::Path};
use try_from_with_context::TryFromWithContext;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Decode, Encode)]
pub struct AccountData {
    uname: String,
    stu_name: String,
    enc_pwd: String,
    login_type: String,
}
impl AccountData {
    #[inline]
    pub fn new(
        uname: String,
        stu_name: String,
        enc_pwd: String,
        login_type: String,
    ) -> AccountData {
        AccountData {
            uname,
            stu_name,
            enc_pwd,
            login_type,
        }
    }
    #[inline]
    pub fn uname(&self) -> &str {
        &self.uname
    }
    #[inline]
    pub fn stu_name(&self) -> &str {
        &self.stu_name
    }
    #[inline]
    pub fn enc_pwd(&self) -> &str {
        &self.enc_pwd
    }
    #[inline]
    pub fn login_type(&self) -> &str {
        &self.login_type
    }
}
impl<UserProtocol: 'static> TryFromWithContext<&AccountDataInternal<UserProtocol>> for AccountData {
    // TODO: use `!` type.
    type Err = ();

    type Context<'cxt> = ();

    #[inline]
    fn try_from<'cxt, P: Borrow<Self::Context<'cxt>>>(
        s: &AccountDataInternal<UserProtocol>,
        _: P,
    ) -> Result<Self, Self::Err> {
        Ok(Self {
            uname: s.uname().to_string(),
            stu_name: s.session.name().to_string(),
            enc_pwd: s.enc_pwd().to_string(),
            login_type: s.login_type().to_owned(),
        })
    }
}
impl<UserProtocol: 'static> TryFromWithContext<AccountDataInternal<UserProtocol>> for AccountData {
    // TODO: use `!` type.
    type Err = ();

    type Context<'cxt> = Path;

    #[inline]
    fn try_from<'cxt, P: Borrow<Self::Context<'cxt>>>(
        account_data: AccountDataInternal<UserProtocol>,
        _: P,
    ) -> Result<Self, Self::Err> {
        let AccountDataInternal {
            session,
            enc_pwd,
            login_solver,
        } = account_data;
        Ok(Self {
            uname: session.uname().to_string(),
            stu_name: session.name().to_string(),
            enc_pwd,
            login_type: login_solver.login_type().to_owned(),
        })
    }
}

impl From<(String, String, String, String)> for AccountData {
    #[inline]
    fn from((uname, stu_name, enc_pwd, login_type): (String, String, String, String)) -> Self {
        Self::new(uname, stu_name, enc_pwd, login_type)
    }
}

impl Display for AccountData {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{},{}", self.uname, self.enc_pwd, self.login_type)
    }
}

impl TryFromWithContext<&str> for AccountData {
    type Err = StoreError;
    type Context<'cxt> = ();

    fn try_from<'cxt, P: Borrow<Self::Context<'cxt>>>(s: &str, _: P) -> Result<Self, Self::Err> {
        let s = s
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();
        if s.len() < 3 {
            Err(StoreError::ParseError(
                "登录所需信息解析出错！格式为 `uname, stu_name, enc_pwd[, login_typ]`.".to_string(),
            ))?
        } else {
            let uname = s[0].to_string();
            let enc_pwd = s[1].to_string();
            let stu_name = s[2].to_string();
            let login_type = if s.len() == 4 {
                s[3].to_string()
            } else {
                String::new()
            };
            let data = AccountData {
                stu_name,
                uname,
                enc_pwd,
                login_type,
            };
            Ok(data)
        }
    }
}
