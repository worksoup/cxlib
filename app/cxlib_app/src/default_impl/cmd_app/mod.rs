mod account;
mod accounts;
mod activity;
mod course;
mod location;
mod locations;
mod where_is_config;

pub use account::*;
pub use accounts::*;
pub use activity::*;
pub use course::*;
use cxlib_store::{AppInfo, Dir, DirTrait};
pub use location::*;
pub use locations::*;
use ref_wrapper::{UNIT, Unit};
pub use where_is_config::*;

#[cfg(feature = "completion")]
mod completions;
#[cfg(feature = "completion")]
pub use completions::*;

use crate::{AliasTable, CourseData, CourseTable, GlobalMultimap, NormalTableTrait};
use clap::Command;
use cxlib_internal::{
    captcha::utils::get_now_timestamp_mills,
    default_impl::{sign::LocationSign, signner::LocationInfoGetterTrait},
    sign::SignTrait,
    types::{
        __private::UnhandledGeoaddr, Course, CourseInfo, Geoaddr, LocationPreprocessorTrait,
        UntypedLoginSolver,
    },
};
use log::warn;
use redb::Database;
use std::cmp;

pub struct CmdAppContext<UserProtocol = cxlib_internal::protocol::collect::UserProtocol> {
    dir: Dir,
    db: Database,
    command: Command,
    login_solvers: GlobalMultimap<UntypedLoginSolver<UserProtocol>>,
    app_info: AppInfo,
}
impl<U> CmdAppContext<U> {
    pub fn new(
        dir: Dir,
        command: Command,
        login_solvers: GlobalMultimap<UntypedLoginSolver<U>>,
        app_info: AppInfo,
    ) -> Self {
        let db = Database::builder().create(dir.get_database_dir()).unwrap();
        Self {
            dir,
            db,
            command,
            app_info,
            login_solvers,
        }
    }
}
impl<U> AsRef<Command> for CmdAppContext<U> {
    fn as_ref(&self) -> &Command {
        &self.command
    }
}
impl<U> AsRef<Database> for CmdAppContext<U> {
    fn as_ref(&self) -> &Database {
        &self.db
    }
}
impl<U> AsRef<AppInfo> for CmdAppContext<U> {
    fn as_ref(&self) -> &AppInfo {
        &self.app_info
    }
}
impl<U> AsRef<GlobalMultimap<UntypedLoginSolver<U>>> for CmdAppContext<U> {
    fn as_ref(&self) -> &GlobalMultimap<UntypedLoginSolver<U>> {
        &self.login_solvers
    }
}
impl<U> AsRef<Unit> for CmdAppContext<U> {
    fn as_ref(&self) -> &Unit {
        &UNIT
    }
}
impl<U> AsRef<Dir> for CmdAppContext<U> {
    fn as_ref(&self) -> &Dir {
        &self.dir
    }
}

pub trait CourseDataFilterAndSorterTrait<Context> {
    fn filter(a: (&Course, &CourseInfo, &CourseData)) -> bool;
    fn sorter(
        a: (&Course, &CourseInfo, &CourseData),
        b: (&Course, &CourseInfo, &CourseData),
    ) -> cmp::Ordering;
}
pub struct DefaultCourseDataSorter;
impl<T> CourseDataFilterAndSorterTrait<T> for DefaultCourseDataSorter {
    fn filter(a: (&Course, &CourseInfo, &CourseData)) -> bool {
        !a.1.ended()
            && (u64::MAX == *a.2.recently_used_timestamp() || {
                let now = (get_now_timestamp_mills() / 1000) as u64;
                *a.2.recently_used_timestamp() > now
                    || now - a.2.recently_used_timestamp() < 160 * 24 * 60 * 60
            })
    }
    fn sorter(
        a: (&Course, &CourseInfo, &CourseData),
        b: (&Course, &CourseInfo, &CourseData),
    ) -> cmp::Ordering {
        let a = a.2.recently_used_timestamp();
        let b = b.2.recently_used_timestamp();
        let now = (get_now_timestamp_mills() / 1000) as u64;
        let da = now > *a && (now - a) / (24 * 60 * 60) == 7;
        let db = now > *b && (now - b) / (24 * 60 * 60) == 7;
        if da == db {
            b.cmp(a)
        } else if da {
            cmp::Ordering::Greater
        } else {
            cmp::Ordering::Less
        }
    }
}

#[derive(Clone, Copy)]
pub struct DefaultLocationInfoGetter<'a>(&'a Database);
impl<'a> DefaultLocationInfoGetter<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self(db)
    }
}
impl<'a> From<&'a Database> for DefaultLocationInfoGetter<'a> {
    fn from(db: &'a Database) -> Self {
        Self::new(db)
    }
}

impl LocationInfoGetterTrait for DefaultLocationInfoGetter<'_> {
    fn get_location_by_location_str(
        &self,
        trimmed_location_str: &str,
        preprocessor: &impl LocationPreprocessorTrait,
    ) -> Option<Geoaddr> {
        // 将字符串作为别名读取数据库，若不存在或读取出错则作为位置解析。
        let r_cxt = self.0.begin_read().ok()?;
        let alias_table = AliasTable::read(&r_cxt).ok()?;
        let geolocation = AliasTable::get_location(&alias_table, trimmed_location_str);
        drop(alias_table);
        match geolocation {
            Ok(addr @ Some(_)) => addr,
            r => {
                if let Err(e) = r {
                    log::warn!("{e:?}");
                }
                let location: Result<UnhandledGeoaddr, _> = trimmed_location_str.parse();
                match location {
                    Ok(location) => Some(location),
                    Err(e) => {
                        warn!("位置字符串无效：`{e}`.");
                        None
                    }
                }
            }
        }
        .map(|l| l.to_location(preprocessor))
    }
    fn get_fallback_location(
        &self,
        sign: &LocationSign,
        preprocessor: &impl LocationPreprocessorTrait,
    ) -> Option<Geoaddr> {
        let r_cxt = self.0.begin_read().ok()?;
        let course_table = CourseTable::read(&r_cxt).ok()?;
        CourseTable::get_course(&course_table, sign.as_inner().course().course())
            .ok()
            .flatten()?
            .1
            .locations()
            .next()
            .cloned()
            .map(|geolocation| geolocation.to_location(preprocessor))
    }
}
