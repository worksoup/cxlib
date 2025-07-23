mod account_data;
mod internal_data;

pub use account_data::*;

use crate::{
    CommonDataTable, GlobalMultimap, ImportExportTrait, KeyType, LoginSolverGetter,
    NormalTableTrait, StoreError, TableDefinitionTrait,
    default_impl::table::{account_table::internal_data::AccountDataInternal, utils::BinCode},
};
use cxlib_error_utils::{CxlibResultUtils, MaybeFatalError};
use cxlib_internal::{
    protocol::collect::UserProtocolTrait,
    types::{LoginError, LoginSolverTrait, Session, UntypedLoginSolver},
};
use log::{error, info, warn};
use redb::{Database, ReadTransaction, ReadableTable, WriteTransaction};
use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    fmt::Display,
    io::Cursor,
    marker::PhantomData,
};
use try_from_with_context::TryFromWithContext;

pub struct AccountTable<UserProtocol>(PhantomData<UserProtocol>);

impl<UserProtocol> AccountTable<UserProtocol> {
    pub fn has_account(
        table: &impl ReadableTable<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
        uid: &str,
    ) -> bool {
        Self::contains_key(table, &uid.to_owned()).unwrap()
    }
    pub fn delete_account(w_cxt: &WriteTransaction, uid: &str) {
        let mut w = <Self as NormalTableTrait>::write(w_cxt).log_unwrap();
        w.remove(&uid.to_owned()).log_unwrap();
    }
    pub fn add_account(
        w_cxt: &WriteTransaction,
        uid: &str,
        account: &AccountData,
    ) -> Result<(), StoreError> {
        let mut w = <Self as NormalTableTrait>::write(w_cxt)?;
        _ = w.insert(uid.to_owned(), account.clone())?;
        Ok(())
    }
    pub fn update_account_and<A: Fn(&WriteTransaction, &str, &AccountData)>(
        w_cxt: &WriteTransaction,
        uid: &str,
        account: &AccountData,
        and: A,
    ) {
        let mut w = <Self as NormalTableTrait>::write(w_cxt).log_unwrap();
        let r = w.insert(uid.to_owned(), account.clone()).log_unwrap();
        if let Some(r) = r {
            and(w_cxt, uid, &r.value())
        }
    }
    pub fn get_all_accounts(
        table: &impl ReadableTable<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
    ) -> HashMap<String, AccountData> {
        table
            .iter()
            .log_unwrap()
            .map(|a| {
                let (k, v) = a.log_unwrap();
                (k.value(), v.value())
            })
            .collect()
    }
    pub fn get_accounts<'a>(
        table: &impl ReadableTable<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
        keys: impl IntoIterator<Item = &'a str>,
    ) -> HashMap<String, AccountData> {
        let keys: HashSet<&str> = keys.into_iter().collect();
        table
            .iter()
            .log_unwrap()
            .filter_map(|a| {
                let (key, data) = a.log_unwrap();
                keys.get(&key.value().as_str())
                    .map(|_| (key.value(), data.value()))
            })
            .collect()
    }
    pub fn get_account(
        table: &impl ReadableTable<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
        uid: &str,
    ) -> Option<AccountData> {
        table.get(&uid.to_owned()).log_unwrap().map(|a| a.value())
    }
}
impl<UserProtocol> AccountTable<UserProtocol>
where
    UserProtocol: 'static + UserProtocolTrait,
{
    fn uid_to_account_data(w_cxt: &WriteTransaction, uid: &str) -> Result<AccountData, StoreError> {
        let table = Self::write(w_cxt).log_unwrap();
        if let Some(account) = Self::get_account(&table, uid) {
            Ok(account)
        } else {
            Err(StoreError::UnexpectedNone(format!(
                "没有该账号：`{uid}`，请检查输入或登录。"
            )))
        }
    }
    fn loop_collect<
        Transaction,
        Cxt: Copy,
        T,
        F: Fn(&Transaction, &str, AccountData, Cxt) -> Result<T, StoreError>,
    >(
        transaction: &Transaction,
        uid_and_account_data: impl IntoIterator<Item = (String, AccountData)>,
        cxt: Cxt,
        f: F,
    ) -> Result<HashMap<String, T>, StoreError> {
        let mut s = HashMap::new();
        for (uid, account_data) in uid_and_account_data {
            match f(transaction, &uid, account_data, cxt) {
                Ok(session) => {
                    s.insert(uid, session);
                }
                Err(e) => {
                    if e.is_fatal() {
                        Err(e)?;
                    } else {
                        warn!("账号加载失败：`{uid}`, `{e}`, 跳过。");
                    }
                }
            }
        }
        Ok(s)
    }
    pub fn load_sessions_by_uid_list_str(
        r_cxt: &ReadTransaction,
        uid_list_str: &str,
    ) -> Result<HashMap<String, Session<UserProtocol>>, StoreError> {
        let str_list = uid_list_str.split(',').map(|a| a.trim());
        let account_data = Self::get_accounts(&Self::read(r_cxt)?, str_list);
        Self::loop_collect(r_cxt, account_data, (), |r_cxt, uid, account_data, _| {
            Self::load_session_internal(r_cxt, uid, &account_data)
        })
    }
    pub fn get_sessions_by_uid_list_str<
        'cxt,
        Cxt: Borrow<<Self as TableDefinitionTrait>::Context<'cxt>>,
    >(
        db: &Database,
        uid_list_str: &str,
        cxt: Cxt,
    ) -> Result<HashMap<String, Session<UserProtocol>>, StoreError> {
        let str_list = uid_list_str.split(',').map(|a| a.trim());
        let r_cxt = db.begin_read()?;
        let account_data = Self::get_accounts(&Self::read(&r_cxt)?, str_list);
        drop(r_cxt);
        Self::loop_collect(db, account_data, cxt.borrow(), Self::get_session_internal)
    }
    #[inline]
    fn none2result(s: impl Display) -> impl FnOnce() -> StoreError {
        move || StoreError::UnexpectedNone(s.to_string())
    }
    #[inline]
    fn get_account_data(
        table: &impl ReadableTable<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
        uid: &str,
    ) -> Result<AccountData, StoreError> {
        if Self::has_account(table, uid) {
            Self::get_account(table, uid).ok_or_else(Self::none2result(format_args!(
                "没有该账号：`{uid}`，请检查输入或登录。"
            )))
        } else {
            Err(StoreError::UnexpectedNone(format!(
                "没有该账号：`{uid}`，请检查输入或登录。"
            )))
        }
    }
    fn load_session_internal(
        r_cxt: &ReadTransaction,
        uid: &str,
        account_data: &AccountData,
    ) -> Result<Session<UserProtocol>, StoreError> {
        let common_data_table = CommonDataTable::read(r_cxt)?;
        let cookies = common_data_table
            .get(&KeyType {
                block: "cookies".to_owned(),
                key: uid.to_owned(),
                identifier: account_data.login_type().to_owned(),
            })?
            .ok_or_else(Self::none2result(format_args!(
                "没有该账号的 cookies 数据：`{uid}`。"
            )))?
            .value();
        Ok(Session::<UserProtocol>::load_cookies(
            account_data.uname(),
            Cursor::new(cookies),
        )?)
    }
    #[inline]
    pub fn load_session(
        r_cxt: &ReadTransaction,
        uid: &str,
    ) -> Result<Session<UserProtocol>, StoreError> {
        let table = AccountTable::<UserProtocol>::read(r_cxt)?;
        let account = Self::get_account_data(&table, uid)?;
        Self::load_session_internal(r_cxt, uid, &account)
    }
    pub fn get_session_internal<
        'cxt,
        Cxt: Borrow<<Self as TableDefinitionTrait>::Context<'cxt>>,
    >(
        db: &Database,
        uid: &str,
        account_data: AccountData,
        cxt: Cxt,
    ) -> Result<Session<UserProtocol>, StoreError> {
        let r_cxt = db.begin_read()?;
        match Self::load_session_internal(&r_cxt, uid, &account_data) {
            Ok(session) => Ok(session),
            Err(e) => match e {
                StoreError::LoginError(LoginError::LoginExpired(_)) => {
                    drop(r_cxt);
                    let w_cxt = db.begin_write()?;
                    warn!(
                        "账号 [{}] 的 cookies 加载失败，尝试重新登录。",
                        account_data.uname()
                    );
                    Ok(
                        <AccountDataInternal<UserProtocol> as TryFromWithContext<_>>::try_from(
                            account_data,
                            (cxt.borrow().clone(), &w_cxt),
                        )?
                        .session,
                    )
                }
                e => {
                    error!("账号 [{}] 的 cookies 加载失败：{e}", account_data.uname());
                    Err(e)?
                }
            },
        }
    }
    #[inline]
    pub fn get_session<'cxt, Cxt: Borrow<<Self as TableDefinitionTrait>::Context<'cxt>>>(
        db: &Database,
        uid: &str,
        cxt: Cxt,
    ) -> Result<Session<UserProtocol>, StoreError> {
        let r_cxt = db.begin_read()?;
        let account = Self::get_account_data(&Self::read(&r_cxt)?, uid)?;
        Self::get_session_internal(db, uid, account, cxt)
    }
    #[inline]
    pub fn load_all_sessions(
        r_cxt: &ReadTransaction,
    ) -> Result<HashMap<String, Session<UserProtocol>>, StoreError> {
        let account_table = AccountTable::<UserProtocol>::read(r_cxt)?;
        let accounts = Self::get_all_accounts(&account_table);
        Self::loop_collect(r_cxt, accounts, (), |w_cxt, uid, account_data, _| {
            Self::load_session_internal(w_cxt, uid, &account_data)
        })
    }
    #[inline]
    pub fn get_all_sessions<'cxt, Cxt: Borrow<<Self as TableDefinitionTrait>::Context<'cxt>>>(
        db: &Database,
        cxt: Cxt,
    ) -> Result<HashMap<String, Session<UserProtocol>>, StoreError> {
        let r_cxt = db.begin_read()?;
        let account_table = AccountTable::<UserProtocol>::read(&r_cxt)?;
        let accounts = Self::get_all_accounts(&account_table);
        Self::loop_collect(db, accounts, cxt.borrow(), Self::get_session_internal)
    }
    #[inline]
    fn store_cookies(
        w_cxt: &WriteTransaction,
        cookies: String,
        uid: &str,
        login_type: &str,
    ) -> Result<(), StoreError> {
        let mut common_data_table = CommonDataTable::write(w_cxt)?;
        common_data_table.insert(
            KeyType {
                block: "cookies".to_owned(),
                key: uid.to_owned(),
                identifier: login_type.to_owned(),
            },
            cookies,
        )?;
        Ok(())
    }
    /// 用于第一次登录。
    pub fn login<'cxt, Cxt: AsRef<<Self as TableDefinitionTrait>::Context<'cxt>>>(
        w_cxt: &WriteTransaction,
        cxt: Cxt,
        uname: String,
        pwd: Option<String>,
        login_type: String,
    ) -> Result<Session<UserProtocol>, StoreError> {
        let cxt = cxt.as_ref();
        let pwd = pwd.ok_or(LoginError::BadPassword("没有密码。".to_owned()))?;
        let solver = LoginSolverGetter::new(cxt, &login_type)
            .ok_or_else(|| LoginError::UnsupportedProtocol)?;
        let login_solver = solver.get_ref();
        let enc_pwd = login_solver.pwd_enc(pwd)?;
        let mut cookies = Cursor::new(Vec::new());
        let session = Session::relogin(&uname, &enc_pwd, &mut cookies, &login_solver)?;
        let cookies_str = String::from_utf8(cookies.into_inner()).unwrap();
        Self::store_cookies(w_cxt, cookies_str, session.uid(), login_solver.login_type())?;
        Self::add_account(
            w_cxt,
            session.uid(),
            &AccountData::new(uname, enc_pwd, login_solver.login_type().to_owned()),
        )?;
        Ok(session)
    }
    fn relogin_internal<'cxt, Cxt: AsRef<<Self as TableDefinitionTrait>::Context<'cxt>>>(
        w_cxt: &WriteTransaction,
        cxt: Cxt,
        account_data: AccountData,
    ) -> Result<Session<UserProtocol>, StoreError> {
        let cxt = cxt.as_ref();
        let solver = LoginSolverGetter::new(cxt, account_data.login_type())
            .ok_or_else(|| LoginError::UnsupportedProtocol)?;
        let solver = solver.get_ref();
        let mut cookies = Cursor::new(Vec::new());
        let uname = account_data.uname();
        let enc_pwd = account_data.enc_pwd();
        let session = Session::relogin(uname, enc_pwd, &mut cookies, &solver)?;
        let cookies_str = String::from_utf8(cookies.into_inner()).unwrap();
        Self::store_cookies(w_cxt, cookies_str, session.uid(), solver.login_type())?;
        Ok(session)
    }
    #[inline]
    pub fn relogin<'cxt, Cxt: AsRef<<Self as TableDefinitionTrait>::Context<'cxt>>>(
        w_cxt: &WriteTransaction,
        cxt: Cxt,
        uid: String,
    ) -> Result<Session<UserProtocol>, StoreError> {
        let account_data = Self::uid_to_account_data(w_cxt, &uid)?;
        Self::relogin_internal(w_cxt, cxt, account_data)
    }
    pub fn relogin_all<'cxt, Cxt: AsRef<<Self as TableDefinitionTrait>::Context<'cxt>>>(
        w_cxt: &WriteTransaction,
        cxt: Cxt,
    ) -> Result<HashMap<String, Session<UserProtocol>>, StoreError> {
        let table = Self::write(w_cxt)?;
        let account_data = Self::get_all_accounts(&table);
        drop(table);
        Self::loop_collect(
            w_cxt,
            account_data,
            &cxt,
            |w_cxt, _, account_data, cxt| Self::relogin_internal(w_cxt, cxt, account_data),
        )
    }
}
impl<UserProtocol> NormalTableTrait for AccountTable<UserProtocol> {}
impl<UserProtocol> TableDefinitionTrait for AccountTable<UserProtocol> {
    type Key = String;
    type Value = BinCode<AccountData>;
    type Context<'cxt> = GlobalMultimap<UntypedLoginSolver<UserProtocol>>;
    const NAME: &'static str = "account";
}
impl<UserProtocol> ImportExportTrait for AccountTable<UserProtocol>
where
    UserProtocol: UserProtocolTrait + 'static,
{
    fn import_text<'cxt, Cxt: Borrow<Self::Context<'cxt>>>(db: &Database, cxt: Cxt, data: &str) {
        let data: Vec<AccountData> =
            crate::default_impl::table::utils::parse_lines::<_, _>(data, ());
        let w_cxt = db.begin_write().log_unwrap();
        for account in data {
            let uname = account.uname();
            match <AccountDataInternal<_> as TryFromWithContext<_>>::try_from(
                account.clone(),
                (cxt.borrow().clone(), &w_cxt),
            ) {
                Ok(data) => {
                    info!(
                        "账号 [{}]（用户名：{}）导入成功！",
                        uname,
                        data.session.name()
                    );
                    Self::add_account(&w_cxt, data.uid(), &account).log_unwrap();
                }
                Err(e) => {
                    warn!("账号 [{uname}] 导入失败！错误信息：{e}.",);
                    continue;
                }
            };
        }
        w_cxt.commit().log_unwrap();
    }

    fn export_text(db: &Database) -> String {
        let r_cxt = db.begin_read().log_unwrap();
        let table = Self::read(&r_cxt).log_unwrap();
        crate::default_impl::table::utils::to_string_lines(Self::get_all_accounts(&table).values())
    }
}
