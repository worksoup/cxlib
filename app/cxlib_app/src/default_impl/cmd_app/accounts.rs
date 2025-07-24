use crate::{AccountTable, AppTrait, CmdMetaAppTrait, GlobalMultimap};
use clap::{ArgMatches, FromArgMatches, Parser, arg};
use cxlib_error_utils::CxlibResultUtils;
use cxlib_internal::{protocol::collect::UserProtocolTrait, types::UntypedLoginSolver};
use redb::Database;
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
    Context: AsRef<GlobalMultimap<UntypedLoginSolver<UserProtocol>>> + AsRef<Database>,
    UserProtocol: UserProtocolTrait + 'static,
{
    type OwnedData = AccountsParser;

    fn run(&self, cxt: &Context, AccountsParser { fresh }: Self::OwnedData) {
        let solver_cxt = cxt.as_ref();
        let db: &Database = cxt.as_ref();
        let sessions = if fresh {
            let w_cxt = db.begin_write().log_unwrap();
            AccountTable::<UserProtocol>::relogin_all(&w_cxt, cxt)
        } else {
            // 列出所有账号。
            AccountTable::get_all_sessions(db, solver_cxt)
        };
        for session in sessions.log_unwrap().into_values() {
            println!("{}, {}, {}", session.uname(), session.name(), session.uid());
        }
    }
}
impl<Context, OwnedData, UserProtocol> CmdMetaAppTrait<Context, OwnedData>
    for AccountsCmdApp<UserProtocol>
where
    Context: AsRef<GlobalMultimap<UntypedLoginSolver<UserProtocol>>> + AsRef<Database> + 'static,
    OwnedData: 'static,
    UserProtocol: UserProtocolTrait + 'static,
{
    fn read_owned_data(&self, _: &Context, matches: &[&ArgMatches]) -> AccountsParser {
        AccountsParser::from_arg_matches(matches.last().unwrap()).unwrap()
    }
}
