use crate::{
    AccountTable, AppTrait, CmdMetaAppTrait, GlobalMultimap, LoginSolverGetter, NormalTableTrait,
};
use clap::{ArgMatches, FromArgMatches, Parser, arg};
use cxlib_error_utils::CxlibResultUtils;
use cxlib_internal::{
    protocol::collect::UserProtocolTrait,
    types::{Session, UntypedLoginSolver},
};
use redb::Database;
use std::{marker::PhantomData, path::Path};

#[derive(Parser, Debug, Clone)]
// TODO: build.rs 中通过环境变量设置 alias.
#[command(name = "accounts", alias = "lsa")]
/// 列出所有账号。
pub struct AccountsParser {
    /// 重新获取账号信息并缓存。
    #[arg(short, long)]
    fresh: bool,
}
pub struct AccountsCmdApp<UserProtocol> {
    _t: PhantomData<UserProtocol>,
}
impl<'cxt, Context, UserProtocol> AppTrait<Context> for AccountsCmdApp<UserProtocol>
where
    Context:
        AsRef<Path> + AsRef<GlobalMultimap<UntypedLoginSolver<UserProtocol>>> + AsRef<Database>,
    UserProtocol: UserProtocolTrait + 'static,
{
    type OwnedData = AccountsParser;

    fn run(&self, cxt: &Context, AccountsParser { fresh }: Self::OwnedData) {
        let solver_cxt = cxt.as_ref();
        let db_path = cxt.as_ref();
        let db: &Database = cxt.as_ref();
        let r_cxt = db.begin_read().log_unwrap();
        let account_table = AccountTable::<UserProtocol>::read(&r_cxt).log_unwrap();
        let sessions: Vec<(Session<UserProtocol>, String)> = if fresh {
            AccountTable::<UserProtocol>::get_all_accounts(&account_table)
                .into_iter()
                .filter_map(|a| {
                    let login_solver = LoginSolverGetter::new(solver_cxt, a.login_type())
                        .unwrap()
                        .get();
                    let mut cookeis = std::io::Cursor::new(Vec::new());
                    let session =
                        Session::relogin(a.uname(), a.enc_pwd(), &mut cookeis, &login_solver);
                    let cookies_str = String::from_utf8(cookeis.into_inner()).unwrap();

                    Some((session.ok()?, cookies_str))
                })
                .collect()
        } else {
            // 列出所有账号。
            AccountTable::get_all_sessions(&account_table, (solver_cxt.clone(), db_path))
                .log_unwrap()
                .into_values()
                .collect()
        };
        for session in sessions {
            println!("{}, {}, {}", session.uname(), session.name(), session.uid());
        }
    }
}
impl<'cxt, Context, OwnedData, UserProtocol> CmdMetaAppTrait<Context, OwnedData>
    for AccountsCmdApp<UserProtocol>
where
    Context: AsRef<Path>
        + AsRef<GlobalMultimap<UntypedLoginSolver<UserProtocol>>>
        + AsRef<Database>
        + 'static,
    OwnedData: 'static,
    UserProtocol: UserProtocolTrait + 'static,
{
    fn read_owned_data(&self, _: &Context, matches: &[&ArgMatches]) -> AccountsParser {
        AccountsParser::from_arg_matches(matches.last().unwrap()).unwrap()
    }
}
