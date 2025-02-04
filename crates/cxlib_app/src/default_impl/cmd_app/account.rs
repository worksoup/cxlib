use crate::{
    cmd_app::{CmdApp, CmdMetaAppTrait},
    AppTrait,
};
use clap::{arg, ArgMatches, Command, CommandFactory, FromArgMatches, Parser, Subcommand};
use cxlib_internal::{
    default_impl::store::{AccountTable, DataBase},
    login::{DefaultLoginSolver, LoginSolverTrait},
};
use log::{info, warn};
// TODO: build.rs 中通过环境变量设置 alias.
#[derive(Parser, Debug, Clone)]
#[command(name = "account", alias = "a")]
/// 账号相关操作（添加、删除）。
pub enum AccountParser {
    /// 添加账号。
    #[command(alias = "+")]
    Add {
        /// 账号（手机号）。
        uname: String,
        /// 密码（明文）。
        /// 指定后将跳过询问密码阶段。
        passwd: Option<String>,
    },
    /// 删除账号。
    #[command(alias = "rm")]
    Remove {
        /// uid (可通过 accounts 子命令查看).
        uid: String,
        /// 无需确认直接删除。
        #[arg(short, long)]
        yes: bool,
    },
}

pub struct AccountCmdApp;
impl<Context: AsRef<DataBase>> AppTrait<Context> for AccountCmdApp {
    type OwnedData = AccountParser;
    fn run(&self, context: &Context, owned_data: Self::OwnedData) {
        match owned_data {
            AccountParser::Add { uname, passwd } => {
                let pwd = cxlib_internal::utils::inquire_pwd(passwd);
                let login_type_and_uname = uname.split_once(":");
                let session = if let Some((login_type, uname)) = login_type_and_uname {
                    AccountTable::login(context.as_ref(), uname.into(), pwd, login_type.into())
                } else {
                    AccountTable::login(
                        context.as_ref(),
                        uname.clone(),
                        pwd,
                        DefaultLoginSolver.login_type().into(),
                    )
                };
                // 添加账号。
                match session {
                    Ok(session) => {
                        info!("添加账号[{uname}]（用户名：{}）成功！", session.name())
                    }
                    Err(e) => warn!("添加账号[{uname}]失败：{e}."),
                };
            }
            AccountParser::Remove { uid, yes } => {
                if !yes {
                    let ans = inquire::Confirm::new("是否删除？")
                        .with_default(false)
                        .prompt()
                        .unwrap_or_else(|e| {
                            warn!("无法识别输入：{e}.");
                            false
                        });
                    if !ans {
                        return;
                    }
                }
                // 删除指定账号。
                AccountTable::delete_account(context.as_ref(), &uid);
            }
        }
    }
}

impl<Context: AsRef<DataBase> + 'static, OwnedData: 'static> CmdMetaAppTrait<Context, OwnedData>
    for AccountCmdApp
{
    fn read_owned_data(
        &self,
        _: &Context,
        matches: &[&ArgMatches],
    ) -> <Self as AppTrait<Context, ()>>::OwnedData {
        AccountParser::from_arg_matches(matches.last().unwrap()).unwrap()
    }
}
