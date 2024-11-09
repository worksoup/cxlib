use crate::store::{DataBase, DataBaseTableTrait};
use cxsign_store::StorageTableCommandTrait;
use cxsign_user::Session;
use log::{info, warn};
use std::collections::HashMap;
use std::fmt::Display;
use std::str::FromStr;

pub struct AccountTable;
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
impl AccountTable {
    pub fn get_sessions_by_accounts_str(db: &DataBase, accounts: &str) -> HashMap<String, Session> {
        let str_list = accounts.split(',').map(|a| a.trim()).collect::<Vec<&str>>();
        let mut s = HashMap::new();
        for account in str_list {
            if let Some(session) = Self::get_session(db, account) {
                s.insert(account.to_string(), session);
            }
        }
        s
    }
    pub fn get_session(db: &DataBase, account: &str) -> Option<Session> {
        if Self::has_account(db, account) {
            Some(Session::load_json(account).unwrap())
        } else {
            warn!("没有该账号：[`{account}`]，请检查输入或登录。");
            None
        }
    }
    pub fn get_sessions(db: &DataBase) -> HashMap<String, Session> {
        let binding = Self::get_accounts(db);
        let str_list = binding.keys().collect::<Vec<_>>();
        let mut s = HashMap::new();
        for account in str_list {
            if Self::has_account(db, &account.uname) {
                let session =
                    Session::load_json_or_relogin(&account.uname, &account.enc_pwd).unwrap();
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
    pub fn has_account(db: &DataBase, uname: &str) -> bool {
        let mut query = db
            .prepare(format!(
                "SELECT count(*) FROM {} WHERE uname=?;",
                Self::TABLE_NAME
            ))
            .unwrap();
        query.bind((1, uname)).unwrap();
        query.next().unwrap();
        query.read::<i64, _>(0).unwrap() > 0
    }

    pub fn delete_account(db: &DataBase, uname: &str) {
        if Self::has_account(db, uname) {
            let mut query = db
                .prepare(format!("DELETE FROM {} WHERE uname=?;", Self::TABLE_NAME))
                .unwrap();
            query.bind((1, uname)).unwrap();
            query.next().unwrap();
        }
        std::fs::remove_file(cxsign_dir::Dir::get_json_file_path(uname)).unwrap();
    }

    pub fn add_account_or<O: Fn(&DataBase, &str, &str, &str)>(
        db: &DataBase,
        uname: &str,
        pwd: &str,
        name: &str,
        or: O,
    ) {
        let mut query = db
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
            Err(_) => or(db, uname, pwd, name),
        };
    }

    pub fn update_account(db: &DataBase, uname: &str, pwd: &str, name: &str) {
        let mut query = db
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

    pub fn get_accounts(db: &DataBase) -> HashMap<UnameAndEncPwdPair, String> {
        let mut query = db
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
    pub fn get_account(db: &DataBase, account: &str) -> Option<(UnameAndEncPwdPair, String)> {
        let mut query = db
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
    /// 用于第一次登录。
    pub fn login(
        db: &DataBase,
        uname: String,
        pwd: Option<String>,
    ) -> Result<Session, cxsign_error::Error> {
        let pwd = pwd.ok_or(cxsign_error::Error::LoginError("没有密码！".to_string()))?;
        let pwd = pwd.as_bytes();
        assert!(pwd.len() > 7);
        assert!(pwd.len() < 17);
        let enc_pwd = cxsign_login::utils::des_enc(pwd);
        let session = Session::relogin(&uname, &enc_pwd)?;
        let name = session.get_stu_name();
        Self::add_account_or(db, &uname, &enc_pwd, name, AccountTable::update_account);
        Ok(session)
    }
    pub fn relogin(db: &DataBase, uname: String) -> Result<Session, cxsign_error::Error> {
        if let Some((UnameAndEncPwdPair { uname, enc_pwd }, _)) =
            AccountTable::get_account(db, &uname)
        {
            let session = Session::relogin(&uname, &enc_pwd)?;
            session.store_json();
            Ok(session)
        } else {
            warn!("数据库中没有该用户！可能是实现错误。");
            panic!()
        }
    }
}

impl StorageTableCommandTrait<DataBase> for AccountTable {
    fn init(storage: &DataBase) {
        <Self as DataBaseTableTrait>::init(storage);
    }
    fn uninit(storage: &DataBase) -> bool {
        !Self::is_existed(storage)
    }
    fn clear(storage: &DataBase) {
        Self::delete(storage);
    }
    fn import(storage: &DataBase, content: &str) {
        <Self as DataBaseTableTrait>::import(storage, content);
    }
    fn export(storage: &DataBase) -> String {
        <Self as DataBaseTableTrait>::export(storage)
    }
}
impl DataBaseTableTrait for AccountTable {
    const TABLE_ARGS: &'static str =
        "uname CHAR (50) UNIQUE NOT NULL,pwd TEXT NOT NULL,name TEXT NOT NULL";
    const TABLE_NAME: &'static str = "account";

    fn import(db: &DataBase, data: &str) {
        db.add_table::<Self>();
        let data = crate::utils::parse::<cxsign_error::Error, UnameAndEncPwdPair>(data);
        for UnameAndEncPwdPair { uname, enc_pwd } in data {
            match Session::relogin(uname.as_str(), &enc_pwd) {
                Ok(session) => {
                    info!(
                        "账号 [{uname}]（用户名：{}）导入成功！",
                        session.get_stu_name()
                    );
                    Self::add_account_or(
                        db,
                        uname.as_str(),
                        &enc_pwd,
                        uname.as_str(),
                        AccountTable::update_account,
                    );
                }
                Err(e) => warn!("账号 [{uname}] 导入失败！错误信息：{e}."),
            }
        }
    }

    fn export(db: &DataBase) -> String {
        crate::utils::to_string(Self::get_accounts(db).keys())
    }
}
