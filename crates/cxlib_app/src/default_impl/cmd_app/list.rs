use crate::{AppTrait, CmdApp, CmdMetaAppTrait, SignParser};
use clap::{ArgMatches, Args, Command, CommandFactory, FromArgMatches, Parser};
use cxlib_internal::types::ext::{ActivityExt, CourseExt};
use cxlib_internal::{
    default_impl::store::{AccountTable, DataBase},
    sign::SignTrait,
    types::{Activity, Course, RawSign, Session},
};
use log::warn;
use std::collections::HashMap;

#[derive(Debug, Parser, Clone)]
// TODO: build.rs 中通过环境变量设置 alias.
#[command(name = "list", alias = "ls")]
/// 列出有效签到。
pub struct ListParser {
    /// 列出指定课程的签到。
    #[arg(short, long)]
    course: Option<i64>,
    /// 列出所有签到（包括无效签到）。
    #[arg(short, long)]
    all: bool,
    /// 从有限的课程中获取。若设置 `-a, --all` 标志则该选项无效。
    #[arg(short, long)]
    limit: Option<u64>,
}
impl ListParser {
    pub fn list_course_activities(
        db: &DataBase,
        course: i64,
        all: bool,
        sessions: HashMap<String, Session>,
    ) {
        let courses = Course::get_from_sessions(sessions.values())
            .unwrap_or_default()
            .into_keys()
            .map(|c| (c.id(), c))
            .collect::<HashMap<_, _>>();
        let (a, n) = courses
            .get(&course)
            .and_then(|course| {
                sessions
                    .values()
                    .next()
                    .and_then(|session| Activity::get_from_courses(db, session, course, true).ok())
            })
            .map(|a| {
                a.into_iter()
                    .filter_map(|k| match k {
                        Activity::RawSign(k) => Some(k),
                        Activity::Other(_) => None,
                    })
                    .partition(|k| k.is_valid())
            })
            .unwrap_or_else(|| (vec![], vec![]));
        // 列出指定课程的有效签到。
        for a in a {
            if a.course.id() == course {
                println!("{}", a.fmt_without_course_info());
            }
        }
        if all {
            // 列出指定课程的所有签到。
            for a in n {
                if a.course.id() == course {
                    println!("{}", a.fmt_without_course_info());
                }
            }
        }
    }
    pub fn list_all_activities(db: &DataBase, all: bool) {
        let sessions = AccountTable::get_sessions(db);
        let activities: HashMap<Activity, Vec<Session>> =
            Activity::get_from_courses(db, sessions.values(), all).unwrap_or_else(|e| {
                warn!("未能获取签到列表，错误信息：{e}.",);
                Default::default()
            });
        let (available_sign_activities, other_sign_activities): (Vec<RawSign>, Vec<RawSign>) =
            activities
                .into_keys()
                .filter_map(|k| match k {
                    Activity::RawSign(k) => Some(k),
                    Activity::Other(_) => None,
                })
                .partition(|a| a.is_valid());
        // 列出所有有效签到。
        for a in available_sign_activities {
            println!("{}", a.as_inner());
        }
        if all {
            // 列出所有签到。
            for a in other_sign_activities {
                println!("{}", a.as_inner());
            }
        } else {
            warn!("{}", SignParser::NOTICE);
        }
    }
}
pub struct ListCmdApp;
impl<Context: AsRef<DataBase>> AppTrait<Context> for ListCmdApp {
    type OwnedData = ListParser;

    fn run(&self, context: &Context, ListParser { course, all, limit }: Self::OwnedData) {
        let sessions = AccountTable::get_sessions(context.as_ref());
        if let Some(course) = course {
            ListParser::list_course_activities(context.as_ref(), course, all, sessions)
        } else {
            ListParser::list_all_activities(context.as_ref(), all)
        }
    }
}
impl<Context: AsRef<DataBase> + 'static, OwnedData: 'static> CmdMetaAppTrait<Context, OwnedData>
    for ListCmdApp
{
    fn read_owned_data(
        &self,
        _: &Context,
        matches: &[&ArgMatches],
    ) -> <Self as AppTrait<Context, ()>>::OwnedData {
        ListParser::from_arg_matches(matches.last().unwrap()).unwrap()
    }
}
