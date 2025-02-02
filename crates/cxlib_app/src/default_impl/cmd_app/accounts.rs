use crate::{AppTrait, CmdApp, CmdMetaAppTrait};
use clap::{arg, ArgMatches, Command, CommandFactory, FromArgMatches, Parser};
use cxlib_internal::{
    default_impl::store::{AccountTable, DataBase},
    login::LoginSolverWrapper,
    types::Session,
};

#[derive(Parser, Debug, Clone)]
// TODO: build.rs 中通过环境变量设置 alias.
#[command(name = "accounts", alias = "lsa")]
/// 列出所有账号。
pub struct AccountsParser {
    /// 重新获取账号信息并缓存。
    #[arg(short, long)]
    fresh: bool,
}
pub struct AccountsCmdApp {
    command: Command,
}

impl AccountsCmdApp {
    pub fn new() -> Self {
        let command = AccountsParser::command();
        Self { command }
    }
}
impl Default for AccountsCmdApp {
    fn default() -> Self {
        Self::new()
    }
}
impl<Context: AsRef<DataBase>> AppTrait<Context> for AccountsCmdApp {
    type OwnedData = AccountsParser;

    fn run(&self, db: &Context, AccountsParser { fresh }: Self::OwnedData) {
        let sessions: Vec<Session> = if fresh {
            AccountTable::get_accounts(db.as_ref())
                .into_iter()
                .filter_map(|a| {
                    let session = Session::relogin(
                        a.uname(),
                        a.enc_pwd(),
                        &LoginSolverWrapper::new(a.login_type()),
                    );
                    session.ok()
                })
                .collect()
        } else {
            // 列出所有账号。
            AccountTable::get_sessions(db.as_ref())
                .into_values()
                .collect()
        };
        for session in sessions {
            println!("{}, {}, {}", session.uname(), session.name(), session.uid());
        }
    }
}
impl<Context: AsRef<DataBase> + 'static> CmdMetaAppTrait<CmdApp<Context>, Context>
    for AccountsCmdApp
{
    fn subcommand(&self) -> Option<&Command> {
        Some(&self.command)
    }
    fn read_owned_data(&self, _: &Context, matches: &[&ArgMatches]) -> AccountsParser {
        AccountsParser::from_arg_matches(matches.last().unwrap()).unwrap()
    }
}
