use crate::{
    AccountTable, AppTrait, CmdMetaAppTrait, GlobalMultimap, database_guard::DatabaseGuard,
};
use clap::{ArgMatches, FromArgMatches, Parser, arg};
use cxlib_error_utils::CxlibResultUtils;
use cxlib_internal::{protocol::collect::UserProtocolTrait, types::UntypedLoginSolver};
use std::marker::PhantomData;

#[derive(Parser, Debug, Clone)]
// TODO: build.rs 中通过环境变量设置 alias.
#[command(name = "accounts", alias = "lsa")]
/// 列出所有账号。
pub struct AccountsParser {
    /// 重新获取账号信息并缓存。
    #[arg(short, long)]
    fresh: bool,
}
pub struct AccountsCmdApp<UserProtocol = cxlib_internal::protocol::collect::UserProtocol>(
    PhantomData<UserProtocol>,
);
impl<U> Default for AccountsCmdApp<U> {
    #[inline]
    fn default() -> Self {
        Self(Default::default())
    }
}
impl<Context, UserProtocol> AppTrait<Context> for AccountsCmdApp<UserProtocol>
where
    Context: AsRef<GlobalMultimap<UntypedLoginSolver<UserProtocol>>> + AsRef<DatabaseGuard>,
    UserProtocol: UserProtocolTrait + 'static,
{
    type OwnedData = AccountsParser;

    fn run(&self, cxt: &Context, AccountsParser { fresh }: Self::OwnedData) {
        let solver_cxt: &GlobalMultimap<UntypedLoginSolver<UserProtocol>> = cxt.as_ref();
        let db_g: &DatabaseGuard = cxt.as_ref();
        let mut db_g = db_g.clone();
        let sessions = if fresh {
            db_g.write_once(|w_cxt| AccountTable::<UserProtocol>::relogin_all(w_cxt, cxt))
                .log_unwrap()
        } else {
            // 列出所有账号。
            AccountTable::get_all_sessions(&mut db_g, solver_cxt).log_unwrap()
        };
        for session in sessions.into_values() {
            println!("{}, {}, {}", session.uname(), session.name(), session.uid());
        }
        // TODO: 更新课程信息。
    }
}
impl<Context, OwnedData, UserProtocol> CmdMetaAppTrait<Context, OwnedData>
    for AccountsCmdApp<UserProtocol>
where
    Context:
        AsRef<GlobalMultimap<UntypedLoginSolver<UserProtocol>>> + AsRef<DatabaseGuard> + 'static,
    OwnedData: 'static,
    UserProtocol: UserProtocolTrait + 'static,
{
    #[inline]
    fn read_owned_data(&self, _: &Context, matches: &[&ArgMatches]) -> AccountsParser {
        AccountsParser::from_arg_matches(matches.last().unwrap()).unwrap()
    }
}
