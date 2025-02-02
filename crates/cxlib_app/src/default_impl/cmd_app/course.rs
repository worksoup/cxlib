use crate::{AppTrait, CmdApp, CmdMetaAppTrait};
use clap::{ArgMatches, Command, CommandFactory, FromArgMatches, Parser};
use cxlib_internal::{
    captcha::utils::get_now_timestamp_mills,
    default_impl::store::{AccountTable, CourseData, CourseTable, DataBase},
    error::CxlibResultUtils,
    types::{ext::CourseExt, Course},
};
use std::collections::HashSet;

#[derive(Parser, Debug, Clone)]
// TODO: build.rs 中通过环境变量设置 alias.
#[command(name = "courses", alias = "lsc")]
/// 获取课程信息。
pub struct CoursesParser {
    /// 待操作账号，格式为以半角逗号隔开的 uid (可通过 accounts 子命令查看).
    #[arg(short, long)]
    uid: Option<String>,
    /// 列出课程之前刷新缓存。
    #[arg(short, long)]
    fresh: bool,
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
        let cached_courses = CourseTable::get_courses(db.as_ref());
        if data.fresh {
            let (sessions, _) = if let Some(uid_list_str) = &data.uid {
                (
                    AccountTable::get_sessions_by_uid_list_str(db.as_ref(), uid_list_str),
                    true,
                )
            } else {
                (AccountTable::get_sessions(db.as_ref()), false)
            };
            // 获取课程信息。
            let courses = Course::get_from_sessions(sessions.values()).unwrap_or_default();
            // 列出所有课程。
            for (c, sessions) in courses {
                println!("{}", c);
                if !cached_courses.contains_key(&c.id()) {
                    CourseTable::insert_course(
                        db.as_ref(),
                        &CourseData::new(
                            c,
                            sessions
                                .iter()
                                .map(|s| s.uid())
                                .collect::<Vec<_>>()
                                .join(","),
                            (get_now_timestamp_mills() / 1000) as u64,
                        ),
                    )
                    .log_unwrap()
                }
            }
        } else if let Some(uid) = &data.uid {
            let set = uid.split(',').map(|s| s.trim()).collect::<HashSet<&str>>();
            for (_, c) in cached_courses.into_iter().filter(|(_, c)| {
                let set_ = c.get_uid_list();
                // 交集不为空。
                !set.is_disjoint(&set_)
            }) {
                println!("{}", c.as_inner());
            }
        } else {
            // 列出所有课程。
            for (_, c) in cached_courses {
                println!("{}", c.as_inner());
            }
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
