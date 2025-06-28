use crate::{GlobalMultimap, StoreError, default_impl::table::account_table::AccountData};
use cxlib_internal::{
    protocol::collect::UserProtocolTrait,
    types::Session,
    types::{LoginError, LoginSolverTrait, UntypedLoginSolver},
};
use std::{borrow::Borrow, fmt::Display, path::Path};
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
    pub fn new<P: AsRef<Path> + ?Sized>(
        uname: String,
        enc_pwd: String,
        store_path: &P,
        login_solver: UntypedLoginSolver<UserProtocol>,
    ) -> Result<Self, LoginError>
    where
        UserProtocol: UserProtocolTrait,
    {
        let session = Session::relogin(&uname, &enc_pwd, store_path, &login_solver)?;
        Ok(Self {
            session: session.into(),
            enc_pwd,
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
impl<P, UserProtocol> TryFrom<(String, String, &P, UntypedLoginSolver<UserProtocol>)>
    for AccountDataInternal<UserProtocol>
where
    P: AsRef<Path> + ?Sized,
    UserProtocol: 'static + UserProtocolTrait,
{
    type Error = LoginError;

    fn try_from(
        (uname, enc_pwd, store_path, login_solver): (
            String,
            String,
            &P,
            UntypedLoginSolver<UserProtocol>,
        ),
    ) -> Result<Self, Self::Error> {
        AccountDataInternal::new(uname, enc_pwd, store_path, login_solver)
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
    type Context<'cxt> = (GlobalMultimap<UntypedLoginSolver<UserProtocol>>, &'cxt Path);
    fn try_from<'cxt, Cxt: Borrow<Self::Context<'cxt>>>(
        account_data: AccountData,
        cxt: Cxt,
    ) -> Result<Self, Self::Err> {
        let (cxt, store_path) = cxt.borrow();
        let solver = cxt.build(account_data.login_type());
        if let Some(solver) = solver {
            let session = Session::relogin(
                account_data.uname(),
                account_data.enc_pwd(),
                store_path,
                &solver,
            )?;
            Ok(Self {
                session,
                enc_pwd: account_data.enc_pwd().to_owned(),
                login_solver: UntypedLoginSolver::from_typed(solver),
            })
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
    type Context<'cxt> = (GlobalMultimap<UntypedLoginSolver<UserProtocol>>, &'cxt Path);

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
