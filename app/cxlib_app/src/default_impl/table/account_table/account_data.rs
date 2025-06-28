use super::internal_data::*;
use crate::StoreError;
use bincode::{Decode, Encode};
use cxlib_internal::types::LoginSolverTrait;
use std::{borrow::Borrow, fmt::Display, path::Path};
use try_from_with_context::TryFromWithContext;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Decode, Encode)]
pub struct AccountData {
    uname: String,
    enc_pwd: String,
    login_type: String,
}
impl AccountData {
    pub fn new(uname: String, enc_pwd: String, login_type: String) -> AccountData {
        AccountData {
            uname,
            enc_pwd,
            login_type,
        }
    }
    pub fn uname(&self) -> &str {
        &self.uname
    }
    pub fn enc_pwd(&self) -> &str {
        &self.enc_pwd
    }
    pub fn login_type(&self) -> &str {
        &self.login_type
    }
}
impl<UserProtocol: 'static> TryFromWithContext<&AccountDataInternal<UserProtocol>> for AccountData {
    // TODO: use `!` type.
    type Err = ();

    type Context<'cxt> = ();

    fn try_from<'cxt, P: Borrow<Self::Context<'cxt>>>(
        s: &AccountDataInternal<UserProtocol>,
        _: P,
    ) -> Result<Self, Self::Err> {
        Ok(Self {
            uname: s.uname().to_string(),
            enc_pwd: s.enc_pwd().to_string(),
            login_type: s.login_type().to_owned(),
        })
    }
}
impl<UserProtocol: 'static> TryFromWithContext<AccountDataInternal<UserProtocol>> for AccountData {
    // TODO: use `!` type.
    type Err = ();

    type Context<'cxt> = Path;

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
            enc_pwd,
            login_type: login_solver.login_type().to_owned(),
        })
    }
}

impl From<(String, String, String)> for AccountData {
    fn from((uname, enc_pwd, login_type): (String, String, String)) -> Self {
        Self::new(uname, enc_pwd, login_type)
    }
}

impl Display for AccountData {
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
        if s.len() < 2 {
            Err(StoreError::ParseError(
                "登录所需信息解析出错！格式为 `uname, enc_pwd[, login_typ]`.".to_string(),
            ))?
        } else {
            let uname = s[0].to_string();
            let enc_pwd = s[1].to_string();
            let login_type = if s.len() == 3 {
                s[2].to_string()
            } else {
                String::new()
            };
            let data = AccountData {
                uname,
                enc_pwd,
                login_type,
            };
            Ok(data)
        }
    }
}
