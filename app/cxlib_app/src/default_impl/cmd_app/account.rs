use crate::{
    AccountTable, AppTrait, CourseData, CourseTable, NormalTableTrait, StoreError,
    TableDefinitionTrait, cmd_app::CmdMetaAppTrait, database_guard::DatabaseGuard,
};
use clap::{ArgMatches, FromArgMatches, Parser, arg};
use cxlib_error_utils::CxlibResultUtils;
use cxlib_internal::{
    protocol::collect::UserProtocolTrait,
    types::{DefaultLoginSolver, LoginSolverTrait},
};
use log::{info, warn};
use std::marker::PhantomData;
/// 支持的登录方式枚举。
///
/// 应当如下实现：
/// ``` rust, no_run
/// #[derive(clap::ValueEnum, Clone)]
/// pub enum DefaultLoginTypeEnum {
///     Default,
///     Custom, // 将会使用其 `name`(默认为蛇形命名法形式) 属性作为 LoginType.
///     #[value(name = "xd")] // 可以通过 clap 的属性改变其 `name` 属性。
///     Other,
/// }
/// ```
pub trait LoginTypeEnumTrait: clap::ValueEnum + Send + Sync + 'static {
    fn login_type(&self) -> String {
        self.to_possible_value().unwrap().get_name().to_owned()
    }
}
impl<T> LoginTypeEnumTrait for T where T: clap::ValueEnum + Send + Sync + 'static {}
#[derive(clap::ValueEnum, Clone)]
pub enum DefaultLoginTypeEnum {
    Default,
    None,
}
// TODO: build.rs 中通过环境变量设置 alias.
#[derive(Parser, Debug, Clone)]
#[command(name = "account", alias = "a")]
/// 账号相关操作（添加、删除）。
pub enum AccountParser<LoginTypeEnum = DefaultLoginTypeEnum>
where
    LoginTypeEnum: LoginTypeEnumTrait,
{
    /// 添加账号。
    #[command(alias = "+")]
    Add {
        /// 账号（手机号）。
        uname: String,
        /// 登录方式。
        #[arg(short, long)]
        login_type: Option<LoginTypeEnum>,
        /// 密码（明文）。
        /// 指定后将跳过询问密码阶段。
        #[arg(short, long)]
        password: Option<String>,
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
pub struct AccountCmdApp<
    LoginTypeEnum = DefaultLoginTypeEnum,
    UserProtocol = cxlib_internal::protocol::collect::UserProtocol,
>(PhantomData<(UserProtocol, LoginTypeEnum)>);
impl<U> Default for AccountCmdApp<U> {
    #[inline]
    fn default() -> Self {
        Self(Default::default())
    }
}
impl<LoginTypeEnum, UserProtocol> AccountCmdApp<LoginTypeEnum, UserProtocol> {
    pub fn add<'cxt, Context>(
        db: &mut DatabaseGuard,
        context: &Context,
        uname: String,
        password: Option<String>,
        login_type: Option<LoginTypeEnum>,
    ) where
        LoginTypeEnum: LoginTypeEnumTrait,
        UserProtocol: UserProtocolTrait + Send + Sync + 'static,
        Context: AsRef<<AccountTable<UserProtocol> as TableDefinitionTrait>::Context<'cxt>>,
    {
        let pwd = cx_interact::inquire_pwd(password);
        let session = db.write_once(|w_cxt| {
            if let Some(login_type) = login_type {
                AccountTable::<UserProtocol>::login(
                    w_cxt,
                    context,
                    uname.clone(),
                    pwd,
                    login_type.login_type(),
                )
            } else {
                AccountTable::login(
                    w_cxt,
                    context,
                    uname.clone(),
                    pwd,
                    DefaultLoginSolver::<UserProtocol>::default()
                        .login_type()
                        .into(),
                )
            }
        });
        // 添加账号。
        match session {
            Ok(session) => {
                info!("添加账号[{uname}]（用户名：{}）成功！", session.name());
                if let Ok(courses) = session.get_courses() {
                    db.write_once(|w_cxt| {
                        let users = vec![session.uid().to_owned()];
                        for course in courses {
                            let data = CourseData::new(None, users.clone(), vec![]);
                            let mut table = CourseTable::write(w_cxt)?;
                            let (course, info) = course.unwrap();
                            CourseTable::merge_course(&mut table, course, (info, data), true)?;
                        }
                        Ok::<_, StoreError>(())
                    })
                    .log_ignore();
                } else {
                    warn!("获取用户[{}]课程失败。", session.name());
                }
            }
            Err(e) => warn!("添加账号[{uname}]失败：{e}."),
        };
    }
    pub fn remove(db: &mut DatabaseGuard, uid: String, yes: bool) {
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
        let old_data = db
            .write_once(|w_cxt| AccountTable::<UserProtocol>::delete_account(w_cxt, &uid))
            .log_unwrap();
        if let Some(old_data) = old_data {
            info!("用户[{}]的信息已删除。", old_data.stu_name())
        } else {
            warn!("指定用户不存在。",)
        }
        // TODO: 删除课程列表中的账号信息。如果账号信息为空，则删除课程。
    }
}
impl<'cxt, Context, LoginTypeEnum: LoginTypeEnumTrait, UserProtocol> AppTrait<Context>
    for AccountCmdApp<LoginTypeEnum, UserProtocol>
where
    Context: AsRef<<AccountTable<UserProtocol> as TableDefinitionTrait>::Context<'cxt>>
        + AsRef<DatabaseGuard>,
    UserProtocol: Send + Sync + UserProtocolTrait + 'static,
{
    type OwnedData = AccountParser<LoginTypeEnum>;
    #[inline]
    fn run(&self, context: &Context, owned_data: Self::OwnedData) {
        let db_g: &DatabaseGuard = context.as_ref();
        let mut db_g = db_g.clone();
        match owned_data {
            AccountParser::Add {
                uname,
                password,
                login_type,
            } => {
                Self::add(&mut db_g, context, uname, password, login_type);
            }
            AccountParser::Remove { uid, yes } => {
                Self::remove(&mut db_g, uid, yes);
            }
        }
    }
}

impl<'cxt, Context, OwnedData, LoginTypeEnum: LoginTypeEnumTrait, UserProtocol>
    CmdMetaAppTrait<Context, OwnedData> for AccountCmdApp<LoginTypeEnum, UserProtocol>
where
    Context: AsRef<<AccountTable<UserProtocol> as TableDefinitionTrait>::Context<'cxt>>
        + AsRef<DatabaseGuard>
        + 'static,
    OwnedData: 'static,
    UserProtocol: Send + Sync + UserProtocolTrait + 'static,
{
    #[inline]
    fn read_owned_data(
        &self,
        _: &Context,
        matches: &[&ArgMatches],
    ) -> <Self as AppTrait<Context, ()>>::OwnedData {
        AccountParser::from_arg_matches(matches.last().unwrap()).unwrap()
    }
}
