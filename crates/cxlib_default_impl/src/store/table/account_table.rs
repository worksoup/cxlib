use crate::store::{DataBase, DataBaseTableTrait};
use cxlib_dir::Dir;
use cxlib_error::Error;
use cxlib_login::{DefaultLoginSolver, LoginSolverTrait, LoginSolverWrapper};
use cxlib_store::StorageTableCommandTrait;
use cxlib_user::Session;
use log::{info, warn};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    str::FromStr,
};
pub struct AccountTable;
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AccountData {
    uid: String,
    uname: String,
    enc_pwd: String,
    login_type: String,
}
impl AccountData {
    pub fn new(uid: String, uname: String, enc_pwd: String, login_type: String) -> AccountData {
        Self {
            uid,
            uname,
            enc_pwd,
            login_type,
        }
    }
    pub fn uid(&self) -> &str {
        &self.uid
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
impl From<(String, String, String)> for AccountData {
    fn from((uid, uname, enc_pwd): (String, String, String)) -> Self {
        AccountData::new(uid, uname, enc_pwd, "".to_owned())
    }
}
impl From<(String, String, String, String)> for AccountData {
    fn from((uid, uname, enc_pwd, login_type): (String, String, String, String)) -> Self {
        AccountData::new(uid, uname, enc_pwd, login_type)
    }
}
impl Display for AccountData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{},{}", self.uname, self.enc_pwd, self.login_type)
    }
}
impl FromStr for AccountData {
    type Err = cxlib_error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();
        if s.len() < 2 {
            Err(Error::ParseError(
                "登录所需信息解析出错！格式为 `uname,enc_pwd[, login_typ]`.".to_string(),
            ))
        } else {
            let uname = s[0].to_string();
            let enc_pwd = s[1].to_string();
            let login_type = if s.len() == 3 {
                s[2].to_string()
            } else {
                String::new()
            };
            let (agent, cookies) =
                Session::relogin_raw(&uname, &enc_pwd, &LoginSolverWrapper::new(&login_type))?;
            Session::store_cookies(&agent, cookies.get_uid())?;
            Ok(Self {
                uid: cookies.get_uid().to_owned(),
                uname,
                enc_pwd,
                login_type,
            })
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
    pub fn get_session(db: &DataBase, uid: &str) -> Option<Session> {
        if Self::has_account(db, uid) {
            let account = Self::get_account(db, uid)?;
            Session::load_cookies(uid, account.uname()).ok()
        } else {
            warn!("没有该账号：[`{uid}`]，请检查输入或登录。");
            None
        }
    }
    pub fn get_sessions(db: &DataBase) -> HashMap<String, Session> {
        let accounts = Self::get_accounts(db).into_iter().collect::<Vec<_>>();
        let mut s = HashMap::new();
        for account in accounts {
            if Self::has_account(db, &account.uid) {
                if let Ok(session) = Session::load_cookies_or_relogin(
                    account.uname(),
                    account.uid(),
                    account.enc_pwd(),
                    &LoginSolverWrapper::new(account.login_type()),
                ) {
                    s.insert(account.uid.clone(), session);
                } else {
                    warn!("账号加载失败：[`{}`]，跳过。", account.uname);
                }
            } else {
                warn!(
                    "没有该账号：[`{}`]，跳过。请检查输入或登录。",
                    account.uname
                );
            }
        }
        s
    }
    pub fn has_account(db: &DataBase, uid: &str) -> bool {
        let mut query = db
            .prepare(format!(
                "SELECT count(*) FROM {} WHERE uid=?;",
                Self::TABLE_NAME
            ))
            .unwrap();
        query.bind((1, uid)).unwrap();
        query.next().unwrap();
        query.read::<i64, _>(0).unwrap() > 0
    }

    pub fn delete_account(db: &DataBase, uid: &str) {
        if Self::has_account(db, uid) {
            let mut query = db
                .prepare(format!("DELETE FROM {} WHERE uid=?;", Self::TABLE_NAME))
                .unwrap();
            query.bind((1, uid)).unwrap();
            query.next().unwrap();
        }
        std::fs::remove_file(Dir::get_json_file_path(uid)).unwrap();
    }

    pub fn add_account_or<O: Fn(&DataBase, &AccountData)>(
        db: &DataBase,
        account: &AccountData,
        or: O,
    ) {
        let mut query = db
            .prepare(format!(
                "INSERT INTO {}(uid,uname,enc_pwd,login_type) values(:uid,:uname,:enc_pwd,:login_type);",
                Self::TABLE_NAME
            ))
            .unwrap();
        query
            .bind::<&[(_, sqlite::Value)]>(
                &[
                    (":uid", account.uid().into()),
                    (":uname", account.uname().into()),
                    (":enc_pwd", account.enc_pwd().into()),
                    (":login_type", account.login_type().into()),
                ][..],
            )
            .unwrap();
        match query.next() {
            Ok(_) => (),
            Err(_) => or(db, account),
        };
    }

    pub fn update_account(db: &DataBase, account: &AccountData) {
        let mut query = db
            .prepare(format!(
                "UPDATE {} SET uname=:uname,enc_pwd=:enc_pwd,login_type=:login_type WHERE uid=:uid;",
                Self::TABLE_NAME
            ))
            .unwrap();
        query
            .bind::<&[(_, sqlite::Value)]>(
                &[
                    (":uid", account.uid().into()),
                    (":uname", account.uname().into()),
                    (":enc_pwd", account.enc_pwd().into()),
                    (":login_type", account.login_type().into()),
                ][..],
            )
            .unwrap();
        query.next().unwrap();
    }

    pub fn get_accounts(db: &DataBase) -> HashSet<AccountData> {
        let mut query = db
            .prepare(format!("SELECT * FROM {};", Self::TABLE_NAME))
            .unwrap();
        let mut accounts = HashSet::new();
        for c in query.iter() {
            if let Ok(row) = c {
                let uid: &str = row.read("uid");
                let uname: &str = row.read("uname");
                let enc_pwd: &str = row.read("enc_pwd");
                let login_type: &str = row.read("login_type");
                accounts.insert(AccountData::new(
                    uid.into(),
                    uname.into(),
                    enc_pwd.into(),
                    login_type.into(),
                ));
            } else {
                warn!("账号解析行出错：{c:?}.");
            }
        }
        if accounts.is_empty() {
            warn!("没有登录的账号，请登录。");
        }
        accounts
    }
    pub fn get_account(db: &DataBase, uid: &str) -> Option<AccountData> {
        let mut query = db
            .prepare(format!("SELECT * FROM {} WHERE uid=?;", Self::TABLE_NAME))
            .unwrap();
        query.bind((1, uid)).unwrap();
        for c in query.iter() {
            if let Ok(row) = c {
                let uid: &str = row.read("uid");
                let uname: &str = row.read("uname");
                let enc_pwd: &str = row.read("enc_pwd");
                let login_type: &str = row.read("login_type");
                return Some(AccountData::new(
                    uid.into(),
                    uname.into(),
                    enc_pwd.into(),
                    login_type.into(),
                ));
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
        login_type: String,
    ) -> Result<Session, cxlib_error::Error> {
        let pwd = pwd.ok_or(Error::LoginError("没有密码！".to_string()))?;
        let solver = LoginSolverWrapper::new(&login_type);
        let enc_pwd = solver.pwd_enc(pwd)?;
        let session = Session::relogin(&uname, &enc_pwd, &LoginSolverWrapper::new(&login_type))?;
        Self::add_account_or(
            db,
            &AccountData::new(session.get_uid().to_owned(), uname, enc_pwd, login_type),
            AccountTable::update_account,
        );
        Ok(session)
    }
    pub fn relogin(db: &DataBase, uid: String) -> Result<Session, cxlib_error::Error> {
        if let Some(AccountData {
            uid,
            uname,
            enc_pwd,
            login_type,
        }) = AccountTable::get_account(db, &uid)
        {
            let session =
                Session::relogin(&uname, &enc_pwd, &LoginSolverWrapper::new(&login_type))?;
            Session::store_cookies(&session, &uid)?;
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
        "uid CHAR (50) UNIQUE NOT NULL,uname TEXT NOT NULL,enc_pwd TEXT NOT NULL,login_type TEXT NOT NULL";
    const TABLE_NAME: &'static str = "account";

    fn import(db: &DataBase, data: &str) {
        db.add_table::<Self>();
        let data = crate::utils::parse::<cxlib_error::Error, AccountData>(data);
        for account in data {
            match Session::relogin(account.uname(), account.enc_pwd(), &DefaultLoginSolver) {
                Ok(session) => {
                    info!(
                        "账号 [{}]（用户名：{}）导入成功！",
                        account.uname(),
                        session.get_stu_name()
                    );
                    Self::add_account_or(db, &account, AccountTable::update_account);
                }
                Err(e) => warn!("账号 [{}] 导入失败！错误信息：{e}.", account.uname(),),
            }
        }
    }

    fn export(db: &DataBase) -> String {
        crate::utils::to_string(Self::get_accounts(db).iter())
    }
}
