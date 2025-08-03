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
use cxlib_error_utils::CxlibResultUtils;
use cxlib_store::{AppInfo, ConfigDir};
pub use location::*;
pub use locations::*;
use ref_wrapper::{UNIT, Unit};
pub use where_is_config::*;

#[cfg(feature = "completion")]
mod completions;
#[cfg(feature = "completion")]
pub use completions::*;

use crate::{
    AliasTable, CourseData, CourseTable, GlobalMultimap, NormalTableTrait,
    database_guard::DatabaseGuard,
};
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
use std::{cmp, path::Path};

pub struct CmdAppContext<UserProtocol = cxlib_internal::protocol::collect::UserProtocol> {
    dir: ConfigDir,
    db: DatabaseGuard,
    command: Command,
    login_solvers: GlobalMultimap<UntypedLoginSolver<UserProtocol>>,
    app_info: AppInfo,
}
impl<U> CmdAppContext<U> {
    pub fn new(
        config_dir: ConfigDir,
        db_file_name: impl AsRef<Path>,
        command: Command,
        login_solvers: GlobalMultimap<UntypedLoginSolver<U>>,
        app_info: AppInfo,
    ) -> Self {
        let db = redb::Database::builder()
            .create(config_dir.get_config_dir().join(db_file_name))
            .unwrap();
        Self {
            dir: config_dir,
            db: DatabaseGuard::new(db),
            command,
            app_info,
            login_solvers,
        }
    }
}
impl<U> AsRef<Command> for CmdAppContext<U> {
    #[inline]
    fn as_ref(&self) -> &Command {
        &self.command
    }
}
impl<U> AsRef<DatabaseGuard> for CmdAppContext<U> {
    #[inline]
    fn as_ref(&self) -> &DatabaseGuard {
        &self.db
    }
}
impl<U> AsRef<AppInfo> for CmdAppContext<U> {
    #[inline]
    fn as_ref(&self) -> &AppInfo {
        &self.app_info
    }
}
impl<U> AsRef<GlobalMultimap<UntypedLoginSolver<U>>> for CmdAppContext<U> {
    #[inline]
    fn as_ref(&self) -> &GlobalMultimap<UntypedLoginSolver<U>> {
        &self.login_solvers
    }
}
impl<U> AsRef<Unit> for CmdAppContext<U> {
    #[inline]
    fn as_ref(&self) -> &Unit {
        &UNIT
    }
}
impl<U> AsRef<ConfigDir> for CmdAppContext<U> {
    #[inline]
    fn as_ref(&self) -> &ConfigDir {
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
    #[inline]
    fn filter(a: (&Course, &CourseInfo, &CourseData)) -> bool {
        !a.1.ended() && {
            if let Some(mills) = a.2.recently_used_timestamp() {
                let now = (get_now_timestamp_mills() / 1000) as u64;
                *mills > now || now - mills < 160 * 24 * 60 * 60
            } else {
                true
            }
        }
    }
    fn sorter(
        a: (&Course, &CourseInfo, &CourseData),
        b: (&Course, &CourseInfo, &CourseData),
    ) -> cmp::Ordering {
        // 默认排序为升序。应该是Greater的靠后。
        match (a.1.ended(), b.1.ended()) {
            (true, true) => {
                let a = a.2.recently_used_timestamp().unwrap_or(u64::MAX);
                let b = b.2.recently_used_timestamp().unwrap_or(u64::MAX);
                let now = (get_now_timestamp_mills() / 1000) as u64;
                let da = now > a && (now - a) / (24 * 60 * 60) == 7;
                let db = now > b && (now - b) / (24 * 60 * 60) == 7;
                if da == db {
                    b.cmp(&a)
                } else if da {
                    cmp::Ordering::Less
                } else {
                    cmp::Ordering::Greater
                }
            }
            (true, false) => cmp::Ordering::Less,
            (false, true) => cmp::Ordering::Greater,
            (false, false) => a.0.cmp(b.0),
        }
    }
}

#[derive(Clone, Copy)]
pub struct DefaultLocationInfoGetter<'a>(&'a DatabaseGuard);
impl<'a> DefaultLocationInfoGetter<'a> {
    #[inline]
    pub fn new(db: &'a DatabaseGuard) -> Self {
        Self(db)
    }
}
impl<'a> From<&'a DatabaseGuard> for DefaultLocationInfoGetter<'a> {
    #[inline]
    fn from(db: &'a DatabaseGuard) -> Self {
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
        let geolocation = self.0.read_once(|r_cxt| {
            let alias_table = AliasTable::read(r_cxt)?;
            AliasTable::get_location(&alias_table, trimmed_location_str)
        });
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
        let r = self
            .0
            .read_once(|r_cxt| {
                let course_table = CourseTable::read(r_cxt)?;
                CourseTable::get_course(&course_table, sign.as_inner().course().course())
            })
            .log_ok()??;
        r.1.locations()
            .next()
            .cloned()
            .map(|geolocation| geolocation.to_location(preprocessor))
    }
}
