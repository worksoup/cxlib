use cxsign_user::Session;

use crate::sql::{DataBase, DataBaseTableTrait};
use log::{info, warn};
use std::collections::HashMap;
use std::fmt::Display;
use std::ops::Deref;
use std::str::FromStr;

pub struct AccountTable<'a> {
    db: &'a DataBase,
}
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnameAndEncPwdPair {
    pub uname: String,
    pub enc_pwd: String,
}
impl From<(String, String)> for UnameAndEncPwdPair {
    fn from((uname, enc_pwd): (String, String)) -> Self {
        UnameAndEncPwdPair { uname, enc_pwd }
    }
}
impl Display for UnameAndEncPwdPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{}", self.uname, self.enc_pwd)
    }
}
impl FromStr for UnameAndEncPwdPair {
    type Err = cxsign_error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();
        if s.len() < 2 {
            Err(cxsign_error::Error::ParseError(
                "登录所需信息解析出错！格式为 `uname,enc_pwd`.".to_string(),
            ))
        } else {
            let uname = s[0].to_string();
            let enc_pwd = s[1].to_string();
            Ok(Self { uname, enc_pwd })
        }
    }
}
impl<'a> AccountTable<'a> {
    pub fn get_sessions_by_accounts_str(&self, accounts: &str) -> HashMap<String, Session> {
        let str_list = accounts.split(',').map(|a| a.trim()).collect::<Vec<&str>>();
        let mut s = HashMap::new();
        for account in str_list {
            if let Some(session) = self.get_session(account) {
                s.insert(account.to_string(), session);
            }
        }
        s
    }
    pub fn get_session(&self, account: &str) -> Option<Session> {
        if self.has_account(account) {
            Some(Session::load_json(account).unwrap())
        } else {
            warn!("没有该账号：[`{account}`]，请检查输入或登录。");
            None
        }
    }
    pub fn get_sessions(&self) -> HashMap<String, Session> {
        let binding = self.get_accounts();
        let str_list = binding.keys().collect::<Vec<_>>();
        let mut s = HashMap::new();
        for account in str_list {
            if self.has_account(&account.uname) {
                let session = Session::load_json(&account.uname).unwrap();
                s.insert(account.uname.clone(), session);
            } else {
                warn!(
                    "没有该账号：[`{}`]，跳过。请检查输入或登录。",
                    account.uname
                );
            }
        }
        s
    }
    pub fn has_account(&self, uname: &str) -> bool {
        let mut query = self
            .db
            .prepare(format!(
                "SELECT count(*) FROM {} WHERE uname=?;",
                Self::TABLE_NAME
            ))
            .unwrap();
        query.bind((1, uname)).unwrap();
        query.next().unwrap();
        query.read::<i64, _>(0).unwrap() > 0
    }

    pub fn delete_account(&self, uname: &str) {
        if self.has_account(uname) {
            let mut query = self
                .db
                .prepare(format!("DELETE FROM {} WHERE uname=?;", Self::TABLE_NAME))
                .unwrap();
            query.bind((1, uname)).unwrap();
            query.next().unwrap();
        }
        std::fs::remove_file(cxsign_dir::Dir::get_json_file_path(uname)).unwrap();
    }

    pub fn add_account_or<O: Fn(&Self, &str, &str, &str)>(
        &self,
        uname: &str,
        pwd: &str,
        name: &str,
        or: O,
    ) {
        let mut query = self
            .db
            .prepare(format!(
                "INSERT INTO {}(uname,pwd,name) values(:uname,:pwd,:name);",
                Self::TABLE_NAME
            ))
            .unwrap();
        query
            .bind::<&[(_, sqlite::Value)]>(
                &[
                    (":pwd", pwd.into()),
                    (":uname", uname.into()),
                    (":name", name.into()),
                ][..],
            )
            .unwrap();
        match query.next() {
            Ok(_) => (),
            Err(_) => or(self, uname, pwd, name),
        };
    }

    pub fn update_account(&self, uname: &str, pwd: &str, name: &str) {
        let mut query = self
            .db
            .prepare(format!(
                "UPDATE {} SET pwd=:pwd,name=:name WHERE uname=:uname;",
                Self::TABLE_NAME
            ))
            .unwrap();
        query
            .bind::<&[(_, sqlite::Value)]>(
                &[
                    (":uname", uname.into()),
                    (":pwd", pwd.into()),
                    (":name", name.into()),
                ][..],
            )
            .unwrap();
        query.next().unwrap();
    }

    pub fn get_accounts(&self) -> HashMap<UnameAndEncPwdPair, String> {
        let mut query = self
            .db
            .prepare(format!("SELECT * FROM {};", Self::TABLE_NAME))
            .unwrap();
        let mut accounts = HashMap::new();
        for c in query.iter() {
            if let Ok(row) = c {
                let uname: &str = row.read("uname");
                let pwd: &str = row.read("pwd");
                let name: &str = row.read("name");
                accounts.insert((uname.into(), pwd.into()).into(), name.into());
            } else {
                warn!("账号解析行出错：{c:?}.");
            }
        }
        if accounts.is_empty() {
            warn!("没有登录的账号，请登录。");
        }
        accounts
    }
    pub fn get_account(&self, account: &str) -> Option<(UnameAndEncPwdPair, String)> {
        let mut query = self
            .db
            .prepare(format!("SELECT * FROM {} WHERE uname=?;", Self::TABLE_NAME))
            .unwrap();
        query.bind((1, account)).unwrap();
        for c in query.iter() {
            if let Ok(row) = c {
                let uname: &str = row.read("uname");
                let pwd: &str = row.read("pwd");
                let name: &str = row.read("name");
                return Some(((uname.into(), pwd.into()).into(), name.into()));
            } else {
                warn!("账号解析行出错：{c:?}.");
            }
        }
        None
    }
    pub fn login(
        &self,
        uname: String,
        pwd: Option<String>,
    ) -> Result<Session, cxsign_error::Error> {
        let pwd = pwd.ok_or(cxsign_error::Error::LoginError("没有密码！".to_string()))?;
        let enc_pwd = cxsign_login::des_enc(&pwd);
        let session = Session::login(&uname, &enc_pwd)?;
        let name = session.get_stu_name();
        self.add_account_or(&uname, &enc_pwd, name, AccountTable::update_account);
        Ok(session)
    }
    pub fn relogin(&self, uname: String, enc_pwd: &str) -> Result<Session, cxsign_error::Error> {
        let session = Session::relogin(&uname, enc_pwd)?;
        self.delete_account(&uname);
        session.store_json();
        let name = session.get_stu_name();
        self.add_account_or(&uname, enc_pwd, name, AccountTable::update_account);
        Ok(session)
    }
}

impl<'a> DataBaseTableTrait<'a> for AccountTable<'a> {
    const TABLE_ARGS: &'static str =
        "uname CHAR (50) UNIQUE NOT NULL,pwd TEXT NOT NULL,name TEXT NOT NULL";
    const TABLE_NAME: &'static str = "account";

    fn from_ref(db: &'a DataBase) -> Self {
        Self { db }
    }

    fn import(db: &'a DataBase, data: String) -> Self {
        let table = db.add_table::<Self>();
        let data = crate::io::parse::<cxsign_error::Error, UnameAndEncPwdPair>(data);
        for UnameAndEncPwdPair { uname, enc_pwd } in data {
            match table.relogin(uname.clone(), &enc_pwd) {
                Ok(session) => info!(
                    "账号 [{uname}]（用户名：{}）导入成功！",
                    session.get_stu_name()
                ),
                Err(e) => warn!("账号 [{uname}] 导入失败！错误信息：{e}."),
            }
        }
        table
    }

    fn export(&self) -> String {
        crate::io::to_string(self.get_accounts().keys())
    }
}

impl<'a> Deref for AccountTable<'a> {
    type Target = DataBase;

    fn deref(&self) -> &Self::Target {
        self.db
    }
}
