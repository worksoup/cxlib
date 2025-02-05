use crate::{AppTrait, CmdMetaAppTrait};
use clap::{ArgMatches, FromArgMatches, Parser};
use cxlib_internal::{
    default_impl::store::{AccountTable, CourseData, CourseTable, DataBase, DataBaseTableTrait},
    error::CxlibResultUtils,
    types::{ext::CourseExt, Course},
};
use std::collections::{HashMap, HashSet};

#[derive(Parser, Debug, Clone)]
// TODO: build.rs 中通过环境变量设置 alias.
#[command(name = "courses", alias = "lsc")]
/// 获取课程信息。
pub struct CoursesParser {
    /// 待操作账号，格式为以半角逗号隔开的 uid (可通过 accounts 子命令查看).
    /// 默认为所有账号。
    #[arg(short, long)]
    uid: Option<String>,
    /// 列出所有课程之前刷新缓存。
    #[arg(short, long)]
    fresh: bool,
}
pub struct CoursesCmdApp;
impl CoursesCmdApp {
    pub fn update_course_table(db: &DataBase) -> HashMap<i64, CourseData> {
        let cached_courses = CourseTable::get_courses(db);
        // 列出所有账号的课程，避免 course.uid_list 不完整。
        let sessions = AccountTable::get_sessions(db);
        // 获取课程信息。
        let courses = Course::get_from_sessions(sessions.values())
            .unwrap_or_default()
            .into_iter()
            .map(|(c, uid_list)| {
                (
                    c.id(),
                    CourseData::new(
                        c.clone(),
                        uid_list
                            .iter()
                            .map(|s| s.uid())
                            .collect::<Vec<_>>()
                            .join(","),
                        // TODO: 应该是没有问题，但是就是有点别扭。
                        // 未出现过的课程时间为 `u64::MAX`.
                        cached_courses
                            .get(&c.id())
                            .map(|c| *c.recently_used_timestamp())
                            .unwrap_or(u64::MAX),
                    ),
                )
            })
            .collect::<HashMap<_, _>>();
        CourseTable::delete(db);
        for (_id, c) in courses.iter() {
            CourseTable::insert_course(db, c).log_unwrap()
        }
        courses
    }
}
impl<Context: AsRef<DataBase>> AppTrait<Context> for CoursesCmdApp {
    type OwnedData = CoursesParser;

    fn run(&self, db: &Context, data: CoursesParser) {
        let courses = if data.fresh {
            Self::update_course_table(db.as_ref())
        } else {
            CourseTable::get_courses(db.as_ref())
        };
        if let Some(uid) = &data.uid {
            let set = uid.split(',').map(|s| s.trim()).collect::<HashSet<&str>>();
            for (_, c) in courses.into_iter().filter(|(_, c)| {
                let set_ = c.get_uid_list();
                // 交集不为空。
                !set.is_disjoint(&set_)
            }) {
                println!("{}", c.as_inner());
            }
        } else {
            // 列出所有课程。
            for (_, c) in courses {
                println!("{}", c.as_inner());
            }
        }
    }
}
impl<Context: AsRef<DataBase> + 'static, OwnedData: 'static> CmdMetaAppTrait<Context, OwnedData>
    for CoursesCmdApp
{
    fn read_owned_data(
        &self,
        _: &Context,
        matches: &[&ArgMatches],
    ) -> <Self as AppTrait<Context, ()>>::OwnedData {
        CoursesParser::from_arg_matches(matches.last().unwrap()).unwrap()
    }
}
