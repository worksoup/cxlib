use crate::{
    AccountTable, AppTrait, CmdMetaAppTrait, CourseData, CourseTable, GlobalMultimap,
    NormalTableTrait, StoreError, TableDefinitionTrait, error,
};
use clap::{ArgMatches, FromArgMatches, Parser};
use cxlib_error_utils::CxlibResultUtils;
use cxlib_internal::{
    protocol::collect::UserProtocolTrait,
    types::{Course, CourseInfo, CourseWithInfo, Session, UntypedLoginSolver, ext::CourseExt},
};
use redb::Database;
use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    marker::PhantomData,
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
    pub fn update_sessions_courses<'a>(
        db: &Database,
        sessions: impl Iterator<Item = &'a Session<UserProtocol>>,
    ) -> Result<HashMap<Course, (CourseInfo, CourseData)>, error::Error>
    where
        UserProtocol: UserProtocolTrait + Send + 'static,
    {
        // 获取课程信息。
        let courses = CourseWithInfo::get_from_sessions(sessions)?;
        let w_cxt = db.begin_write().map_err(StoreError::from)?;
        let mut course_table = CourseTable::write(&w_cxt)?;
        let mut updated = HashMap::new();
        for (course, users) in courses.iter() {
            let course_data = (
                course.info().clone(),
                CourseData::new(
                    u64::MAX,
                    users.iter().map(|s| s.uid().to_owned()).collect(),
                    vec![],
                ),
            );
            let r =
                CourseTable::merge_course(&mut course_table, course.course(), course_data, false)?;
            updated.insert(course.course().clone(), r);
        }
        drop(course_table);
        w_cxt.commit().log_unwrap();
        Ok(updated)
    }
    #[inline]
    pub fn update_course_table<Cxt>(
        db: &Database,
        cxt: Cxt,
    ) -> Result<HashMap<Course, (CourseInfo, CourseData)>, error::Error>
    where
        UserProtocol: UserProtocolTrait + Send + 'static,
        Cxt: Borrow<<AccountTable<UserProtocol> as TableDefinitionTrait>::Context<'cxt>>,
    {
        // 列出所有账号的课程，避免 course.uid_list 不完整。
        let sessions = AccountTable::get_all_sessions(db, cxt)?;
        Self::update_sessions_courses(db, sessions.values())
    }
}
impl<U> Default for CoursesCmdApp<U> {
    #[inline]
    fn default() -> Self {
        Self(Default::default())
    }
}
impl<UserProtocol, Context> AppTrait<Context> for CoursesCmdApp<UserProtocol>
where
    UserProtocol: UserProtocolTrait + std::marker::Send + 'static,
    Context: AsRef<Database> + AsRef<GlobalMultimap<UntypedLoginSolver<UserProtocol>>>,
{
    type OwnedData = CoursesParser;

    fn run(&self, cxt: &Context, data: CoursesParser) {
        // TODO: 刷新时删除原表。
        let courses = if data.fresh {
            let login_solvers: &GlobalMultimap<_> = cxt.as_ref();
            Self::update_course_table(cxt.as_ref(), login_solvers.clone()).log_unwrap_or_default()
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
}
impl<UserProtocol, Context, OwnedData> CmdMetaAppTrait<Context, OwnedData>
    for CoursesCmdApp<UserProtocol>
where
    UserProtocol: std::marker::Send + UserProtocolTrait + 'static,
    Context: AsRef<Database> + AsRef<GlobalMultimap<UntypedLoginSolver<UserProtocol>>> + 'static,
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
