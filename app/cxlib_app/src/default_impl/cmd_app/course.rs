use crate::{
    AccountTable, AppTrait, CmdMetaAppTrait, CourseData, CourseTable, GlobalMultimap,
    NormalTableTrait, StoreError, TableDefinitionTrait, error,
};
use clap::{ArgMatches, FromArgMatches, Parser};
use cxlib_error_utils::CxlibResultUtils;
use cxlib_internal::{
    protocol::collect::UserProtocolTrait,
    types::{Course, CourseInfo, CourseWithInfo, UntypedLoginSolver, ext::CourseExt},
};
use redb::Database;
use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    marker::PhantomData,
    path::Path,
};

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
pub struct CoursesCmdApp<UserProtocol = cxlib_internal::protocol::collect::UserProtocol>(
    PhantomData<UserProtocol>,
);
impl<'cxt, UserProtocol> CoursesCmdApp<UserProtocol> {
    pub fn update_course_table<Cxt>(
        db: &Database,
        cxt: Cxt,
    ) -> Result<HashMap<Course, (CourseInfo, CourseData)>, error::Error>
    where
        UserProtocol: UserProtocolTrait + Send + 'static,
        Cxt: Borrow<<AccountTable<UserProtocol> as TableDefinitionTrait>::Context<'cxt>>,
    {
        let r_cxt = db.begin_read().map_err(StoreError::from)?;
        let course_table = CourseTable::read(&r_cxt)?;
        let cached_courses = CourseTable::get_courses(&course_table)?;
        drop(course_table);
        let account_table = AccountTable::<UserProtocol>::read(&r_cxt)?;
        // 列出所有账号的课程，避免 course.uid_list 不完整。
        let sessions = AccountTable::get_all_sessions(&account_table, cxt);
        // 获取课程信息。
        let courses = CourseWithInfo::get_from_sessions(sessions?.values())?
            .into_iter()
            .map(|(course_with_info, sessions)| {
                let old_data = cached_courses.get(&course_with_info);
                let users = sessions
                    .into_iter()
                    .map(|s| s.uid().to_owned())
                    .collect::<Vec<_>>();
                let (course, info) = course_with_info.unwrap();
                if let Some((recently_used_timestamp, locations)) =
                    old_data.map(|(_, data)| (data.recently_used_timestamp(), data.locations()))
                {
                    (
                        course,
                        (
                            info,
                            CourseData::new(
                                *recently_used_timestamp,
                                users,
                                locations.cloned().collect(),
                            ),
                        ),
                    )
                } else {
                    (
                        course,
                        (
                            info,
                            CourseData::new(
                                // TODO: 应该是没有问题，但是就是有点别扭。
                                // 未出现过的课程时间为 `u64::MAX`.
                                u64::MAX,
                                users,
                                vec![],
                            ),
                        ),
                    )
                }
            })
            .collect::<HashMap<_, _>>();
        drop(r_cxt);
        let w_cxt = db.begin_write().map_err(StoreError::from)?;
        CourseTable::delete(&w_cxt)?;
        let mut course_table = CourseTable::write(&w_cxt)?;
        for (course, (info, data)) in courses.iter() {
            CourseTable::insert_course(&mut course_table, course, (info.clone(), data.clone()))
                .log_unwrap();
        }
        drop(course_table);
        w_cxt.commit().log_unwrap();
        Ok(courses)
    }
}
impl<U> Default for CoursesCmdApp<U> {
    fn default() -> Self {
        Self(Default::default())
    }
}
impl<'cxt, UserProtocol, Context> AppTrait<Context> for CoursesCmdApp<UserProtocol>
where
    UserProtocol: UserProtocolTrait + std::marker::Send + 'static,
    Context:
        AsRef<Database> + AsRef<Path> + AsRef<GlobalMultimap<UntypedLoginSolver<UserProtocol>>>,
{
    type OwnedData = CoursesParser;

    fn run(&self, cxt: &Context, data: CoursesParser) {
        let courses = if data.fresh {
            let (login_solvers, store_path): (&GlobalMultimap<_>, &Path) =
                (cxt.as_ref(), cxt.as_ref());
            let account_table_cxt = (login_solvers.clone(), store_path);
            Self::update_course_table(cxt.as_ref(), account_table_cxt).log_unwrap_or_default()
        } else {
            let db: &Database = cxt.as_ref();
            let r_cxt = db.begin_read().log_unwrap();
            let course_table = CourseTable::read(&r_cxt).log_unwrap();
            CourseTable::get_courses(&course_table).unwrap_or_default()
        };
        if let Some(uid) = &data.uid {
            let set = uid.split(',').map(|s| s.trim()).collect::<HashSet<&str>>();
            for (course, (info, _)) in courses.into_iter().filter(|(_, (_, data))| {
                let set_ = data.users().collect::<HashSet<_>>();
                // 交集不为空。
                !set.is_disjoint(&set_)
            }) {
                println!("{}", CourseWithInfo::new(course, info));
            }
        } else {
            // 列出所有课程。
            for (course, (info, _)) in courses {
                println!("{}", CourseWithInfo::new(course, info));
            }
        }
    }

    fn meta_app<MetaApp: crate::MetaAppTrait<Self, Context, ()>>(self, meta_app: MetaApp) -> Self
    where
        Self: Sized,
    {
        meta_app.register(self)
    }
}
impl<'cxt, UserProtocol, Context, OwnedData> CmdMetaAppTrait<Context, OwnedData>
    for CoursesCmdApp<UserProtocol>
where
    UserProtocol: std::marker::Send + UserProtocolTrait + 'static,
    Context: AsRef<Database>
        + AsRef<Path>
        + AsRef<GlobalMultimap<UntypedLoginSolver<UserProtocol>>>
        + 'static,
    OwnedData: 'static,
{
    fn read_owned_data(
        &self,
        _: &Context,
        matches: &[&ArgMatches],
    ) -> <Self as AppTrait<Context, ()>>::OwnedData {
        CoursesParser::from_arg_matches(matches.last().unwrap()).unwrap()
    }
}
