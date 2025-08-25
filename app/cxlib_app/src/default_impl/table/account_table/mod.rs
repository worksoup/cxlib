mod account_data;
mod internal_data;

pub use account_data::*;

use crate::{
    CommonDataTable, GlobalMultimap, ImportExportTrait, KeyType, LoginSolverGetter,
    NormalTableTrait, StoreError, TableDefinitionTrait,
    database_guard::DatabaseGuard,
    default_impl::table::{account_table::internal_data::AccountDataInternal, utils::BinCode},
};
use cxlib_error_utils::{CxlibResultUtils, MaybeFatalError};
use cxlib_internal::{
    protocol::collect::UserProtocolTrait,
    types::{LoginError, LoginSolverTrait, Session, UntypedLoginSolver},
};
use log::{error, info, warn};
use redb::{ReadTransaction, ReadableTable, TableError, WriteTransaction};
use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    fmt::Display,
    io::Cursor,
    marker::PhantomData,
};
use try_from_with_context::TryFromWithContext;

// TODO: update api.
pub struct AccountTable<UserProtocol>(PhantomData<UserProtocol>);

impl<UserProtocol> AccountTable<UserProtocol> {
    pub fn has_account(
        table: &impl ReadableTable<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
        uid: &str,
    ) -> Result<bool, StoreError> {
        Self::contains_key(table, &uid.to_owned())
    }
    pub fn delete_account(
        w_cxt: &WriteTransaction,
        uid: &str,
    ) -> Result<Option<AccountData>, StoreError> {
        let mut w = <Self as NormalTableTrait>::write(w_cxt)?;
        let v = w.remove(&uid.to_owned())?;
        Ok(v.map(|v| v.value()))
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
trait LoopCollect {
    type SelfType<'a>;
    fn loop_collect_base<
        TransactionRef,
        Cxt: Copy,
        T,
        F: Fn(TransactionRef, &str, AccountData, Cxt) -> Result<T, StoreError>,
    >(
        s: &mut HashMap<String, T>,
        transaction: TransactionRef,
        uid: String,
        account_data: AccountData,
        cxt: Cxt,
        f: F,
    ) -> Result<(), StoreError> {
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
        Ok(())
    }
    fn loop_collect<
        Cxt: Copy,
        T,
        F: Fn(Self::SelfType<'_>, &str, AccountData, Cxt) -> Result<T, StoreError>,
    >(
        self,
        uid_and_account_data: impl IntoIterator<Item = (String, AccountData)>,
        cxt: Cxt,
        f: F,
    ) -> Result<HashMap<String, T>, StoreError>;
}
impl<P: 'static> LoopCollect for &mut P {
    type SelfType<'a> = &'a mut P;

    #[inline]
    fn loop_collect<
        Cxt: Copy,
        T,
        F: Fn(Self::SelfType<'_>, &str, AccountData, Cxt) -> Result<T, StoreError>,
    >(
        self,
        uid_and_account_data: impl IntoIterator<Item = (String, AccountData)>,
        cxt: Cxt,
        f: F,
    ) -> Result<HashMap<String, T>, StoreError> {
        let mut s = HashMap::new();
        for (uid, account_data) in uid_and_account_data {
            Self::loop_collect_base(&mut s, &mut *self, uid, account_data, cxt, &f)?;
        }
        Ok(s)
    }
}
impl<P: 'static> LoopCollect for &P {
    type SelfType<'a> = &'a P;

    #[inline]
    fn loop_collect<
        Cxt: Copy,
        T,
        F: Fn(Self::SelfType<'_>, &str, AccountData, Cxt) -> Result<T, StoreError>,
    >(
        self,
        uid_and_account_data: impl IntoIterator<Item = (String, AccountData)>,
        cxt: Cxt,
        f: F,
    ) -> Result<HashMap<String, T>, StoreError> {
        let mut s = HashMap::new();
        for (uid, account_data) in uid_and_account_data {
            Self::loop_collect_base(&mut s, self, uid, account_data, cxt, &f)?;
        }
        Ok(s)
    }
}
impl<U> AccountTable<U> {
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
    pub fn load_sessions_by_uid_list_str(
        r_cxt: &ReadTransaction,
        uid_list_str: &str,
    ) -> Result<HashMap<String, Session>, StoreError> {
        let str_list = uid_list_str.split(',').map(|a| a.trim());
        let account_data = Self::get_accounts(&Self::read(r_cxt)?, str_list);
        LoopCollect::loop_collect(r_cxt, account_data, (), |r_cxt, uid, account_data, _| {
            Self::load_session_internal(r_cxt, uid, &account_data)
        })
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
        if Self::has_account(table, uid)? {
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
    ) -> Result<Session, StoreError> {
        let common_data_table = CommonDataTable::read(r_cxt)?;
        let cookies = common_data_table
            .get(&KeyType {
                block: "cookies".to_owned(),
                identifier: account_data.login_type().to_owned(),
                key: uid.to_owned(),
            })?
            .ok_or_else(Self::none2result(format_args!(
                "没有该账号的 cookies 数据：`{uid}`。"
            )))?
            .value();
        Ok(Session::load_cookies(
            account_data.uname().to_owned(),
            account_data.stu_name().to_owned(),
            Cursor::new(cookies),
        )?)
    }
    #[inline]
    pub fn load_session(r_cxt: &ReadTransaction, uid: &str) -> Result<Session, StoreError> {
        let table = AccountTable::<U>::read(r_cxt)?;
        let account = Self::get_account_data(&table, uid)?;
        Self::load_session_internal(r_cxt, uid, &account)
    }
    #[inline]
    pub fn load_all_sessions(
        r_cxt: &ReadTransaction,
    ) -> Result<HashMap<String, Session>, StoreError> {
        let account_table = AccountTable::<U>::read(r_cxt)?;
        let accounts = Self::get_all_accounts(&account_table);
        LoopCollect::loop_collect(r_cxt, accounts, (), |w_cxt, uid, account_data, _| {
            Self::load_session_internal(w_cxt, uid, &account_data)
        })
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
                identifier: login_type.to_owned(),
                key: uid.to_owned(),
            },
            cookies,
        )?;
        Ok(())
    }
}
impl<UserProtocol> AccountTable<UserProtocol> {
    pub fn get_sessions_by_uid_list_str<
        'cxt,
        Cxt: Borrow<<Self as TableDefinitionTrait>::Context<'cxt>>,
    >(
        db: &mut DatabaseGuard,
        uid_list_str: &str,
        cxt: Cxt,
    ) -> Result<HashMap<String, Session>, StoreError>
    where
        UserProtocol: UserProtocolTrait + 'static,
    {
        let str_list = uid_list_str.split(',').map(|a| a.trim());
        let account_table = db
            .read(AccountTable::<UserProtocol>::read)
            .map(|account_table| account_table.unwrap_inner());
        let account_table = match account_table {
            Ok(account_table) => account_table,
            Err(StoreError::TableError(TableError::TableDoesNotExist(e))) => {
                warn!("数据表不存在：{e}。");
                db.write_once(|w_cxt| {
                    AccountTable::<UserProtocol>::write(w_cxt)?;
                    Ok::<_, StoreError>(())
                })?;
                db.read(AccountTable::<UserProtocol>::read)?.unwrap_inner()
            }
            Err(e) => Err(e)?,
        };
        let accounts = Self::get_accounts(&account_table, str_list);
        LoopCollect::loop_collect(db, accounts, cxt.borrow(), Self::get_session_internal)
    }
    pub fn get_session_internal<'cxt, Cxt: Borrow<<Self as TableDefinitionTrait>::Context<'cxt>>>(
        db: &mut DatabaseGuard,
        uid: &str,
        account_data: AccountData,
        cxt: Cxt,
    ) -> Result<Session, StoreError>
    where
        UserProtocol: UserProtocolTrait + 'static,
    {
        let session = db.read_once(|r_cxt| Self::load_session_internal(r_cxt, uid, &account_data));

        match session {
            Ok(session) => Ok(session),
            Err(e) => match e {
                StoreError::LoginError(LoginError::LoginExpired(_)) => db.write_once(|w_cxt| {
                    warn!(
                        "账号 [{}] 的 cookies 加载失败，尝试重新登录。",
                        account_data.uname()
                    );
                    Ok(
                        <AccountDataInternal<UserProtocol> as TryFromWithContext<_>>::try_from(
                            account_data,
                            (cxt.borrow().clone(), w_cxt),
                        )?
                        .session,
                    )
                }),
                e => {
                    error!("账号 [{}] 的 cookies 加载失败：{e}", account_data.uname());
                    Err(e)?
                }
            },
        }
    }
    #[inline]
    pub fn get_session<'cxt, Cxt: Borrow<<Self as TableDefinitionTrait>::Context<'cxt>>>(
        db: &mut DatabaseGuard,
        uid: &str,
        cxt: Cxt,
    ) -> Result<Session, StoreError>
    where
        UserProtocol: UserProtocolTrait + 'static,
    {
        let account = db.read_once(|r_cxt| Self::get_account_data(&Self::read(r_cxt)?, uid))?;
        Self::get_session_internal(db, uid, account, cxt)
    }
    #[inline]
    pub fn get_all_sessions<'cxt, Cxt: Borrow<<Self as TableDefinitionTrait>::Context<'cxt>>>(
        db: &mut DatabaseGuard,
        cxt: Cxt,
    ) -> Result<HashMap<String, Session>, StoreError>
    where
        UserProtocol: UserProtocolTrait + 'static,
    {
        let account_table = db
            .read(AccountTable::<UserProtocol>::read)
            .map(|account_table| account_table.unwrap_inner());
        let account_table = match account_table {
            Ok(account_table) => account_table,
            Err(StoreError::TableError(TableError::TableDoesNotExist(e))) => {
                warn!("数据表不存在：{e}。");
                db.write_once(|w_cxt| {
                    AccountTable::<UserProtocol>::write(w_cxt)?;
                    Ok::<_, StoreError>(())
                })?;
                db.read(AccountTable::<UserProtocol>::read)?.unwrap_inner()
            }
            Err(e) => Err(e)?,
        };
        let accounts = Self::get_all_accounts(&account_table);
        LoopCollect::loop_collect(db, accounts, cxt.borrow(), Self::get_session_internal)
    }
    /// 用于第一次登录。
    pub fn login<'cxt, Cxt: AsRef<<Self as TableDefinitionTrait>::Context<'cxt>>>(
        w_cxt: &WriteTransaction,
        cxt: Cxt,
        uname: String,
        pwd: Option<String>,
        login_type: String,
    ) -> Result<Session, StoreError>
    where
        UserProtocol: UserProtocolTrait + 'static,
    {
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
            &AccountData::new(
                uname,
                session.name().to_owned(),
                enc_pwd,
                login_solver.login_type().to_owned(),
            ),
        )?;
        Ok(session)
    }
    fn relogin_internal<'cxt, Cxt: AsRef<<Self as TableDefinitionTrait>::Context<'cxt>>>(
        w_cxt: &WriteTransaction,
        cxt: Cxt,
        account_data: AccountData,
    ) -> Result<Session, StoreError>
    where
        UserProtocol: UserProtocolTrait + 'static,
    {
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
    ) -> Result<Session, StoreError>
    where
        UserProtocol: UserProtocolTrait + 'static,
    {
        let account_data = Self::uid_to_account_data(w_cxt, &uid)?;
        Self::relogin_internal(w_cxt, cxt, account_data)
    }
    pub fn relogin_all<'cxt, Cxt: AsRef<<Self as TableDefinitionTrait>::Context<'cxt>>>(
        w_cxt: &WriteTransaction,
        cxt: Cxt,
    ) -> Result<HashMap<String, Session>, StoreError>
    where
        UserProtocol: UserProtocolTrait + 'static,
    {
        let table = Self::write(w_cxt)?;
        let account_data = Self::get_all_accounts(&table);
        drop(table);
        LoopCollect::loop_collect(w_cxt, account_data, &cxt, |w_cxt, _, account_data, cxt| {
            Self::relogin_internal(w_cxt, cxt, account_data)
        })
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
    fn import_text<'cxt, Cxt: Borrow<Self::Context<'cxt>>>(
        db: &mut DatabaseGuard,
        cxt: Cxt,
        data: &str,
    ) {
        let data: Vec<AccountData> =
            crate::default_impl::table::utils::parse_lines::<_, _>(data, ());
        db.write_once(|w_cxt| {
            for account in data {
                let uname = account.uname();
                match <AccountDataInternal<_> as TryFromWithContext<_>>::try_from(
                    account.clone(),
                    (cxt.borrow().clone(), w_cxt),
                ) {
                    Ok(data) => {
                        info!(
                            "账号 [{}]（用户名：{}）导入成功！",
                            uname,
                            data.session.name()
                        );
                        Self::add_account(w_cxt, data.uid(), &account).log_unwrap();
                    }
                    Err(e) => {
                        if e.is_fatal() {
                            return Err(e);
                        }
                        warn!("账号 [{uname}] 导入失败！错误信息：{e}.",);
                        continue;
                    }
                };
            }
            Ok(())
        })
        .log_unwrap();
    }

    fn export_text(db: &mut DatabaseGuard) -> String {
        db.read_once(|r_cxt| {
            let table = Self::read(r_cxt)?;
            Ok::<_, StoreError>(crate::default_impl::table::utils::to_string_lines(
                Self::get_all_accounts(&table).values(),
            ))
        })
        .log_unwrap()
    }
}
