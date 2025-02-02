use crate::{AppTrait, CmdApp, CmdMetaAppTrait};
use clap::{ArgMatches, Command, CommandFactory, FromArgMatches, Parser};
use cxlib_internal::{
    default_impl::store::{AccountTable, DataBase},
    types::Course,
};
#[derive(Parser, Debug, Clone)]
// TODO: build.rs 中通过环境变量设置 alias.
#[command(name = "courses", alias = "lsc")]
/// 获取课程信息。
pub struct CoursesParser {
    /// 待操作账号，格式为以半角逗号隔开的 uid (可通过 accounts 子命令查看).
    #[arg(short, long)]
    uid: Option<String>,
}
pub struct CoursesCmdApp {
    command: Command,
}
impl CoursesCmdApp {
    pub fn new() -> Self {
        Self {
            command: CoursesParser::command(),
        }
    }
}
impl Default for CoursesCmdApp {
    fn default() -> Self {
        Self::new()
    }
}
impl<Context: AsRef<DataBase>> AppTrait<Context> for CoursesCmdApp {
    type OwnedData = CoursesParser;

    fn run(&self, db: &Context, data: CoursesParser) {
        let (sessions, _) = if let Some(uid_list_str) = &data.uid {
            (
                AccountTable::get_sessions_by_uid_list_str(db.as_ref(), uid_list_str),
                true,
            )
        } else {
            (AccountTable::get_sessions(db.as_ref()), false)
        };
        // 获取课程信息。
        let courses = Course::get_courses(sessions.values()).unwrap_or_default();
        // 列出所有课程。
        for (c, _) in courses {
            println!("{}", c);
        }
    }
}
impl<Context: AsRef<DataBase> + 'static> CmdMetaAppTrait<CmdApp<Context>, Context>
    for CoursesCmdApp
{
    fn subcommand(&self) -> Option<&Command> {
        Some(&self.command)
    }

    fn read_owned_data(
        &self,
        _: &Context,
        matches: &[&ArgMatches],
    ) -> <Self as AppTrait<Context, ()>>::OwnedData {
        CoursesParser::from_arg_matches(matches.last().unwrap()).unwrap()
    }
}
