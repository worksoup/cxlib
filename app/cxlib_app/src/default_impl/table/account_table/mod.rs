mod account_data;
mod internal_data;

pub use account_data::*;

use crate::{
    GlobalMultimap, ImportExportTrait, LoginSolverGetter, NormalTableTrait, StoreError,
    TableDefinitionTrait,
    database_guard::DatabaseGuard,
    default_impl::table::{account_table::internal_data::AccountDataInternal, utils::BinCode},
};
use cxlib_error_utils::{CxlibResultUtils, MaybeFatalError};
use cxlib_internal::{
    protocol::collect::UserProtocolTrait,
    types::{LoginError, LoginSolverTrait, Session, UntypedLoginSolver},
};
use log::{error, info, warn};
use redb::{Database, ReadableTable, WriteTransaction};
use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    marker::PhantomData,
    path::Path,
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
    pub fn get_accounts(
        table: &impl ReadableTable<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
    ) -> HashSet<AccountData> {
        table
            .iter()
            .log_unwrap()
            .map(|a| a.log_unwrap().1.value())
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
    pub fn get_sessions_by_uid_list_str(
        table: &impl ReadableTable<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
        uid_list_str: &str,
    ) -> HashMap<String, Session<UserProtocol>> {
        let str_list = uid_list_str
            .split(',')
            .map(|a| a.trim())
            .collect::<Vec<&str>>();
        let mut s = HashMap::new();
        for uid in str_list {
            if let Some(session) = Self::get_session(table, uid) {
                s.insert(uid.to_string(), session);
            }
        }
        s
    }
    pub fn get_session(
        table: &impl ReadableTable<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
        uid: &str,
    ) -> Option<Session<UserProtocol>> {
        // NOTE: read 实际上不会造成错误，除了锁中毒。
        if Self::has_account(table, uid) {
            let account = Self::get_account(table, uid)?;
            Session::<UserProtocol>::load_cookies(uid, account.uname()).ok()
        } else {
            warn!("没有该账号：[`{uid}`]，请检查输入或登录。");
            None
        }
    }
    pub fn get_sessions<'cxt, Cxt: Borrow<<Self as TableDefinitionTrait>::Context<'cxt>>>(
        table: &impl ReadableTable<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
        cxt: Cxt,
    ) -> Result<HashMap<String, Session<UserProtocol>>, StoreError> {
        let accounts = Self::get_accounts(table).into_iter().collect::<Vec<_>>();
        let mut s = HashMap::new();
        for account in accounts {
            let Ok(account) =
                <AccountDataInternal<_> as TryFromWithContext<_>>::try_from(account, cxt.borrow())
            else {
                continue;
            };
            // 应该正在读取，数据库不会被修改，所以不需要再判断是否存在。
            match Session::load_cookies_or_relogin(
                account.uname(),
                account.uid(),
                account.enc_pwd(),
                account.login_solver(),
            ) {
                Ok(session) => {
                    s.insert(account.uid().to_owned(), session);
                }
                Err(e) => {
                    if e.is_fatal() {
                        Err(e)?
                    } else {
                        warn!("账号加载失败：`{}`, `{e}`, 跳过。", account.uname());
                    }
                }
            }
        }
        Ok(s)
    }
    /// 用于第一次登录。
    pub fn login<'cxt, Cxt: AsRef<<Self as TableDefinitionTrait>::Context<'cxt>>>(
        db: &Database,
        cxt: Cxt,
        uname: String,
        pwd: Option<String>,
        login_type: String,
    ) -> Result<Session<UserProtocol>, StoreError> {
        let (cxt, store_path) = cxt.as_ref();
        let pwd = pwd.ok_or(LoginError::BadPassword("没有密码。".to_owned()))?;
        let solver = LoginSolverGetter::new(cxt, &login_type)
            .ok_or_else(|| LoginError::UnsupportedProtocol)?;
        let solver = solver.get_ref();
        let enc_pwd = solver.pwd_enc(pwd)?;
        let session = Session::<UserProtocol>::relogin(&uname, &enc_pwd, store_path, &solver)?;
        let mut g = DatabaseGuard::new(db);
        g.write(|w_cxt| {
            Self::add_account(
                w_cxt,
                session.uid(),
                &AccountData::new(uname, enc_pwd, solver.login_type().to_owned()),
            )
        })?;
        let w_cxt = db.begin_write().log_unwrap();
        w_cxt.commit().log_unwrap();
        Ok(session)
    }
    pub fn relogin<'cxt, Cxt: AsRef<<Self as TableDefinitionTrait>::Context<'cxt>>>(
        table: &impl ReadableTable<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
        cxt: Cxt,
        uid: String,
    ) -> Result<Session<UserProtocol>, StoreError> {
        let Some(account) = AccountTable::<UserProtocol>::get_account(table, &uid) else {
            error!("数据库中没有该用户！可能是实现错误。");
            panic!()
        };
        let (cxt, store_path) = cxt.as_ref();
        let solver = LoginSolverGetter::new(cxt, account.login_type())
            .ok_or_else(|| LoginError::UnsupportedProtocol)?;
        let solver = solver.get_ref();
        let session = Session::<UserProtocol>::relogin(
            account.uname(),
            account.enc_pwd(),
            store_path,
            &solver,
        )?;
        Session::<UserProtocol>::store_cookies(&session, &uid)?;
        Ok(session)
    }
}
impl<UserProtocol> NormalTableTrait for AccountTable<UserProtocol> {}
impl<UserProtocol> TableDefinitionTrait for AccountTable<UserProtocol> {
    type Key = String;
    type Value = BinCode<AccountData>;
    type Context<'cxt> = (GlobalMultimap<UntypedLoginSolver<UserProtocol>>, &'cxt Path);
    const NAME: &'static str = "account";
}
impl<UserProtocol> ImportExportTrait for AccountTable<UserProtocol>
where
    UserProtocol: UserProtocolTrait + 'static,
{
    fn import_text<'cxt, Cxt: Borrow<Self::Context<'cxt>>>(db: &Database, cxt: Cxt, data: &str) {
        let (_, path) = cxt.borrow();
        let data: Vec<AccountData> =
            crate::default_impl::table::utils::parse_lines::<_, _>(data, ());
        for account in data {
            let uname = account.uname();
            match <AccountDataInternal<_> as TryFromWithContext<_>>::try_from(
                account.clone(),
                cxt.borrow(),
            ) {
                Ok(data) => {
                    let solver = data.login_solver();
                    match Session::<UserProtocol>::relogin(uname, account.enc_pwd(), path, solver) {
                        Ok(session) => {
                            info!("账号 [{}]（用户名：{}）导入成功！", uname, session.name());
                            let w_cxt = db.begin_write().log_unwrap();
                            Self::add_account(&w_cxt, data.uid(), &account).log_unwrap();
                            w_cxt.commit().log_unwrap();
                        }
                        Err(e) => warn!("账号 [{}] 导入失败！错误信息：{e}.", account.uname(),),
                    }
                }
                Err(e) => {
                    warn!("账号 [{uname}] 导入失败！错误信息：{e}.",);
                    continue;
                }
            };
        }
    }

    fn export_text(db: &Database) -> String {
        let r_cxt = db.begin_read().log_unwrap();
        let table = Self::read(&r_cxt).log_unwrap();
        crate::default_impl::table::utils::to_string_lines(Self::get_accounts(&table).iter())
    }
}
