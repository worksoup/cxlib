use crate::{
    CommonDataTable, GlobalMultimap, KeyType, NormalTableTrait, StoreError,
    default_impl::table::account_table::AccountData,
};
use cxlib_internal::{
    protocol::collect::UserProtocolTrait,
    types::Session,
    types::{LoginSolverTrait, UntypedLoginSolver},
};
use redb::WriteTransaction;
use std::{borrow::Borrow, fmt::Display, io::Cursor};
use try_from_with_context::TryFromWithContext;

pub struct AccountDataInternal<UserProtocol> {
    pub(in crate::default_impl::table::account_table) session: Session<UserProtocol>,
    pub(in crate::default_impl::table::account_table) enc_pwd: String,
    pub(in crate::default_impl::table::account_table) login_solver:
        UntypedLoginSolver<UserProtocol>,
}
impl<UserProtocol> AccountDataInternal<UserProtocol>
where
    UserProtocol: 'static,
{
    pub fn new(
        uname: &str,
        enc_pwd: &str,
        login_solver: UntypedLoginSolver<UserProtocol>,
        w_cxt: &WriteTransaction,
    ) -> Result<Self, StoreError>
    where
        UserProtocol: UserProtocolTrait,
    {
        let mut cookies = Cursor::new(Vec::new());
        let session = Session::relogin(&uname, &enc_pwd, &mut cookies, &login_solver)?;
        let cookies_str = String::from_utf8(cookies.into_inner()).unwrap();
        let mut common_data_table = CommonDataTable::write(w_cxt)?;
        common_data_table.insert(
            KeyType {
                block: "cookies".to_owned(),
                key: session.uid().to_owned(),
                identifier: login_solver.login_type().to_owned(),
            },
            cookies_str,
        )?;
        Ok(Self {
            session: session.into(),
            enc_pwd: enc_pwd.to_owned(),
            login_solver,
        })
    }
}
impl<UserProtocol> AccountDataInternal<UserProtocol> {
    pub fn uid(&self) -> &str {
        self.session.uid()
    }
    pub fn uname(&self) -> &str {
        self.session.uname()
    }
    pub fn name(&self) -> &str {
        self.session.name()
    }
    pub fn enc_pwd(&self) -> &str {
        &self.enc_pwd
    }
    pub fn login_type(&self) -> &str
    where
        UserProtocol: 'static,
    {
        self.login_solver.login_type()
    }
    pub fn login_solver(&self) -> &UntypedLoginSolver<UserProtocol>
    where
        UserProtocol: 'static,
    {
        &self.login_solver
    }
}
impl<UserProtocol: 'static> Display for AccountDataInternal<UserProtocol> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        AccountData::fmt(
            &<AccountData as TryFromWithContext<_>>::try_from(self, ()).unwrap(),
            f,
        )
    }
}
impl<UserProtocol> TryFromWithContext<AccountData> for AccountDataInternal<UserProtocol>
where
    UserProtocol: UserProtocolTrait + 'static,
{
    type Err = StoreError;
    type Context<'cxt> = (
        GlobalMultimap<UntypedLoginSolver<UserProtocol>>,
        &'cxt WriteTransaction,
    );
    fn try_from<'cxt, Cxt: Borrow<Self::Context<'cxt>>>(
        account_data: AccountData,
        cxt: Cxt,
    ) -> Result<Self, Self::Err> {
        let (cxt, w_cxt) = cxt.borrow();
        let solver = cxt.build(account_data.login_type());
        if let Some(solver) = solver {
            Self::new(account_data.uname(), account_data.enc_pwd(), solver, w_cxt)
        } else {
            Err(StoreError::ParseError(
                "该登录类型未注册，无法持久化。".to_string(),
            ))
        }
    }
}
impl<UserProtocol> TryFromWithContext<&str> for AccountDataInternal<UserProtocol>
where
    UserProtocol: UserProtocolTrait + 'static,
{
    type Err = StoreError;
    type Context<'cxt> = (
        GlobalMultimap<UntypedLoginSolver<UserProtocol>>,
        &'cxt WriteTransaction,
    );

    fn try_from<'cxt, Cxt: Borrow<Self::Context<'cxt>>>(
        s: &str,
        cxt: Cxt,
    ) -> Result<Self, Self::Err> {
        <Self as TryFromWithContext<AccountData>>::try_from(
            <AccountData as TryFromWithContext<&str>>::try_from(s, ())?,
            cxt,
        )
    }
}
