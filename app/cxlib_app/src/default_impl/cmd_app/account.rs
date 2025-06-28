use crate::{AccountTable, AppTrait, TableDefinitionTrait, cmd_app::CmdMetaAppTrait};
use clap::{ArgMatches, FromArgMatches, Parser, arg};
use cxlib_error_utils::CxlibResultUtils;
use cxlib_internal::{
    protocol::collect::UserProtocolTrait,
    types::{DefaultLoginSolver, LoginSolverTrait},
};
use log::{info, warn};
use redb::Database;
use std::marker::PhantomData;

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

pub struct AccountCmdApp<UserProtocol> {
    _t: PhantomData<UserProtocol>,
}
impl<'cxt, Context, UserProtocol> AppTrait<Context> for AccountCmdApp<UserProtocol>
where
    Context: AsRef<<AccountTable<UserProtocol> as TableDefinitionTrait>::Context<'cxt>>
        + AsRef<Database>,
    UserProtocol: Send + Sync + UserProtocolTrait + 'static,
{
    type OwnedData = AccountParser;
    fn run(&self, context: &Context, owned_data: Self::OwnedData) {
        match owned_data {
            AccountParser::Add { uname, passwd } => {
                let pwd = cx_interact::inquire_pwd(passwd);
                let login_type_and_uname = uname.split_once(":");
                let session = if let Some((login_type, uname)) = login_type_and_uname {
                    AccountTable::login(
                        context.as_ref(),
                        context,
                        uname.into(),
                        pwd,
                        login_type.into(),
                    )
                } else {
                    AccountTable::login(
                        context.as_ref(),
                        context,
                        uname.clone(),
                        pwd,
                        DefaultLoginSolver::<UserProtocol>::default()
                            .login_type()
                            .into(),
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
                let db: &Database = context.as_ref();
                // 删除指定账号。
                let w_cxt = db.begin_write().log_unwrap();
                AccountTable::<UserProtocol>::delete_account(&w_cxt, &uid);
                w_cxt.commit().log_unwrap();
            }
        }
    }
}

impl<'cxt, Context, OwnedData, UserProtocol> CmdMetaAppTrait<Context, OwnedData>
    for AccountCmdApp<UserProtocol>
where
    Context: AsRef<<AccountTable<UserProtocol> as TableDefinitionTrait>::Context<'cxt>>
        + AsRef<Database>
        + 'static,
    OwnedData: 'static,
    UserProtocol: Send + Sync + UserProtocolTrait + 'static,
{
    fn read_owned_data(
        &self,
        _: &Context,
        matches: &[&ArgMatches],
    ) -> <Self as AppTrait<Context, ()>>::OwnedData {
        AccountParser::from_arg_matches(matches.last().unwrap()).unwrap()
    }
}
