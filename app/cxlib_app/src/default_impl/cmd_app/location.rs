use crate::{
    AccountTable, AliasTable, AppTrait, CourseTable, GlobalMultimap, ImportExportTrait,
    LocationTable, NormalTableTrait, StoreError, TableDefinitionTrait, cmd_app::CmdMetaAppTrait,
    database_guard::DatabaseGuard,
};
use clap::{ArgMatches, FromArgMatches, Parser, Subcommand, arg};
use cxlib_error_utils::{CxlibResultUtils, MaybeFatalError};
use cxlib_internal::{
    protocol::collect::{TypesProtocolTrait, UserProtocolTrait},
    types::{
        __private::UnhandledGeoaddr, Course, CourseWithInfo, Geolocation, UntypedLoginSolver,
        ext::CourseExt,
    },
};
use cxlib_store::AppInfo;
use log::{error, warn};
use redb::Database;
use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    marker::PhantomData,
    path::PathBuf,
    str::FromStr,
    sync::Arc,
};
#[derive(Debug, Clone)]
pub enum GeolocationOrUnhandledGeoaddr {
    Geolocation(Geolocation),
    UnhandledGeoaddr(UnhandledGeoaddr),
}
impl GeolocationOrUnhandledGeoaddr {
    pub fn into_geolocation(self) -> Geolocation {
        match self {
            GeolocationOrUnhandledGeoaddr::Geolocation(geolocation) => geolocation,
            GeolocationOrUnhandledGeoaddr::UnhandledGeoaddr(unhandled_geoaddr) => {
                unhandled_geoaddr.geolocation
            }
        }
    }
}
impl FromStr for GeolocationOrUnhandledGeoaddr {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<UnhandledGeoaddr>() {
            Ok(ok) => Ok(Self::UnhandledGeoaddr(ok)),
            Err(_) => Ok(Self::Geolocation(s.parse::<Geolocation>()?)),
        }
    }
}
#[derive(Parser, Debug, Clone)]
// TODO: build.rs 中通过环境变量设置 alias.
#[command(name = "location", alias = "l")]
/// 位置相关操作（添加、删除、批量删除、导入、导出）。
// TODO: 需要重新设计。文档需要更新。
pub enum LocationParser {
    /// 添加位置或别名。
    #[command(alias = "+")]
    // TODO: 也许需要添加默认别名。
    Add {
        /// 地址名称、经纬度与海拔。
        /// 格式为：`addr,lon,lat,alt`.
        /// 格式为：`地址,经度,纬度,海拔`.
        location_str: String,
        /// 为位置添加别名。
        alias: Option<String>,
        /// 绑定该位置到指定课程。
        /// 默认添加为全局位置。
        #[arg(short, long)]
        course: Option<Course>,
    },
    #[command(alias = "rm")]
    /// 删除位置。
    Remove {
        #[command(subcommand)]
        command: Remove,
        /// 无需确认直接删除。
        #[arg(short, long)]
        yes: bool,
    },
    #[command(alias = "rd")]
    /// 批量删除位置。
    Reduce {
        #[command(subcommand)]
        reduce_type: ReduceType,
        /// 无需确认直接删除。
        #[arg(short, long)]
        yes: bool,
        /// 指定全部。
        #[arg(short, long)]
        all: bool,
        /// 指定课程。
        #[arg(short, long)]
        course: Option<Course>,
        /// 指定全局。
        #[arg(short, long)]
        global: bool,
    },
    #[command(alias = "i")]
    /// 导入位置。
    Import {
        /// 导入位置。
        /// 每行一个位置。课程号在前，位置在后，最后是别名。它们由字符 `$` 隔开。
        /// 其中位置的格式为 `地址,经度,纬度,海拔`, 别名的格式为以 `/` 分隔的字符串数组。
        #[arg(short, long)]
        input: Option<PathBuf>,
        /// 从班级历史签到中获取位置并导入。格式同上。
        #[arg(short, long)]
        course: Option<Course>,
    },
    #[command(alias = "e")]
    /// 导出位置。
    Export {
        /// 导出位置。
        /// 每行一个位置。课程号在前，位置在后，最后是别名。它们由字符 `$` 隔开。
        /// 其中位置的格式为 `地址,经度,纬度,海拔`, 别名的格式为以 `/` 分隔的字符串数组。
        /// 无法解析的行将会被跳过。
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// 从班级历史签到中获取位置并导出。格式同上。
        #[arg(short, long)]
        course: Option<Course>,
    },
}
// TODO: 需要重新设计。
#[derive(Subcommand, Debug, Clone)]
pub enum ReduceType {
    /// 位置。
    Locations,
    /// 位置别名。
    Aliases {
        /// 位置。
        location: Option<GeolocationOrUnhandledGeoaddr>,
    },
}
#[derive(Subcommand, Debug, Clone)]
pub enum Remove {
    Locations {
        /// 位置。
        #[arg(short, long)]
        location: Option<GeolocationOrUnhandledGeoaddr>,
        /// 位置别名对应的位置。
        #[arg(short, long)]
        alias: Option<String>,
    },
    Aliases {
        /// 位置别名。
        alias: String,
    },
}
impl LocationParser {
    fn confirm(msg: &str) -> bool {
        inquire::Confirm::new(msg)
            .with_default(false)
            .prompt()
            .unwrap_or_else(|e| {
                warn!("无法识别输入：{e}.");
                false
            })
    }
    /// 添加位置或别名。
    fn add(
        app_info: &AppInfo,
        database_guard: &mut DatabaseGuard,
        location_str: String,
        alias: Option<String>,
        course: Option<Course>,
    ) {
        if let Some(course) = course.as_ref() {
            if course.invalid() {
                if course.is_global_course() {
                    warn!("警告：为课程号与班级号均为 -1 的课程设置的位置将被视为全局位置！");
                } else {
                    error!("错误：课程号小于 0! 请检查是否正确！");
                    panic!()
                }
            }
        }
        let location = location_str.parse::<UnhandledGeoaddr>();
        let location = if let Ok(location) = location {
            let contains = database_guard.read_once(|r_cxt| {
                let location_table = LocationTable::read(r_cxt)?;
                LocationTable::has_location(&location_table, &location)
            });
            if let Ok(contains) = contains
                && contains
            {
                if let Some(course) = course {
                    let mut courses = match database_guard.read_once(|r_cxt| {
                        let location_table = LocationTable::read(r_cxt)?;
                        LocationTable::get_courses(&location_table, &location)
                    }) {
                        Ok(courses) => courses,
                        Err(e) => {
                            warn!("`{e}`.");
                            Vec::new()
                        }
                    }
                    .into_iter()
                    .collect::<HashSet<_>>();
                    courses.insert(course);
                    database_guard
                        .write_once(|w_cxt| {
                            let UnhandledGeoaddr {
                                unhandled_place_name,
                                geolocation,
                            } = location.clone();
                            LocationTable::add_location(
                                w_cxt,
                                geolocation,
                                unhandled_place_name,
                                courses.into_iter().collect::<Vec<_>>(),
                            )
                        })
                        .log_unwrap();
                } else {
                    return;
                }
            } else {
                database_guard
                    .write_once(|w_cxt| {
                        let UnhandledGeoaddr {
                            unhandled_place_name,
                            geolocation,
                        } = location.clone();
                        LocationTable::add_location(
                            w_cxt,
                            geolocation,
                            unhandled_place_name,
                            vec![],
                        )
                    })
                    .log_unwrap();
            };
            location
        } else if alias.is_none() {
            warn!("无法确定所要操作的位置对象！");
            return;
        }
        // 无法解释为 `location_id` 则解释为别名。
        else {
            let location = database_guard
                .read(|r_cxt| {
                    let alias_table = AliasTable::read(r_cxt)?;
                    AliasTable::get_location(&alias_table, location_str.trim())
                })
                .log_unwrap();
            if let Some(location) = location.unwrap_inner() {
                location
            } else {
                warn!("无法确定所要操作的位置对象！");
                return;
            }
        };
        if let Some(alias) = alias {
            database_guard
                .write_once(|w_cxt| {
                    let mut alias_table = AliasTable::write(w_cxt)?;
                    AliasTable::update_alias_and(&mut alias_table, &alias, &location, |_w, a, l| {
                        let app = app_info.application();
                        warn!(
                            r#"
该别名 `{a}` 代表的位置已更新为 `{location}`。
如需保留旧位置，请使用 `{app} location add {l} other_alias` 命令重新添加该位置。"#,
                        );
                    })
                })
                .log_unwrap();
        }
    }
    fn remove(database_guard: &mut DatabaseGuard, command: Remove, yes: bool) {
        if !yes {
            let ans = Self::confirm("警告：是否删除？");
            if !ans {
                return;
            }
        }
        match command {
            Remove::Locations { location, alias } => {
                let location = location
                    .map(|location| location.into_geolocation())
                    .or_else(|| {
                        alias.and_then(|alias| {
                            database_guard
                                .read(|r_cxt| {
                                    let alias_table = AliasTable::read(r_cxt)?;
                                    Ok::<_, StoreError>(
                                        AliasTable::get_location(&alias_table, &alias)?
                                            .map(|l| l.geolocation),
                                    )
                                })
                                .log_unwrap()
                                .unwrap_inner()
                        })
                    });
                if let Some(location) = location {
                    let (contains, g) = database_guard
                        .read(|r_cxt| {
                            let location_table = LocationTable::read(r_cxt)?;
                            LocationTable::has_location(&location_table, &location)
                        })
                        .log_unwrap()
                        .into_inner();
                    if contains {
                        let aliases = g.read_once(|r_cxt| {
                            let alias_table = AliasTable::read(r_cxt)?;
                            AliasTable::get_aliases(&alias_table, &location)
                        });
                        let aliases = aliases.log_unwrap_or_default();
                        database_guard
                            .write_once(|w_cxt| LocationTable::delete_location(w_cxt, &location))
                            .log_unwrap();
                        for alias in aliases.iter() {
                            database_guard
                                .write_once(|w_cxt| {
                                    let mut alias_table = AliasTable::write(w_cxt)?;
                                    AliasTable::delete_alias(&mut alias_table, alias)
                                })
                                .log_unwrap();
                        }
                    } else {
                        warn!("警告：未指定有效的位置，将不做任何事情。");
                    }
                } else {
                    warn!("警告：未指定有效的位置，将不做任何事情。");
                }
            }
            Remove::Aliases { alias } => {
                let contains = database_guard
                    .read_once(|r_cxt| {
                        let alias_table = AliasTable::read(r_cxt)?;
                        AliasTable::has_alias(&alias_table, &alias)
                    })
                    .log_unwrap();
                if contains {
                    database_guard
                        .write_once(|w_cxt| {
                            let mut alias_table = AliasTable::write(w_cxt)?;
                            AliasTable::delete_alias(&mut alias_table, &alias)
                        })
                        .log_unwrap();
                } else {
                    warn!("警告：该别名并不存在，将不做任何事情。");
                }
            }
        }
    }
    fn reduce(
        database_guard: &mut DatabaseGuard,
        reduce_type: ReduceType,
        yes: bool,
        all: bool,
        course: Option<Course>,
        global: bool,
    ) {
        if !yes {
            let ans = Self::confirm("警告：是否删除？");
            if !ans {
                return;
            }
        }
        let course = course.or(if global {
            Some(Course::global_course())
        } else {
            None
        });
        let mut locations: Vec<Geolocation> = if let Some(course) = course {
            let course = database_guard
                .read(|r_cxt| {
                    let course_table = CourseTable::read(r_cxt)?;
                    CourseTable::get_course(&course_table, &course)
                })
                .log_unwrap()
                .unwrap_inner();
            if let Some((_, data)) = course {
                data.decompose()
                    .2
                    .into_iter()
                    .map(|UnhandledGeoaddr { geolocation, .. }| geolocation)
                    .collect()
            } else {
                Default::default()
            }
        } else if all {
            database_guard
                .read(|r_cxt| {
                    let location_table = LocationTable::read(r_cxt)?;
                    LocationTable::get_locations(&location_table)
                })
                .log_unwrap()
                .unwrap_inner()
                .into_keys()
                .collect()
        } else {
            Default::default()
        };
        let delete_locations = match reduce_type {
            ReduceType::Locations => true,
            ReduceType::Aliases { location } => {
                if let Some(location) = location {
                    locations = vec![location.into_geolocation()]
                }
                false
            }
        };

        if locations.is_empty() {
            if delete_locations {
                warn!("警告：未指定任何有效的位置，将不做任何事情。");
            } else {
                warn!("警告：未指定任何有效的别名，将不做任何事情。");
            }
            return;
        }
        let mut aliases = Vec::new();
        for location in locations.iter() {
            let aliases_ = database_guard
                .read(|r_cxt| {
                    let alias_table = AliasTable::read(r_cxt)?;
                    AliasTable::get_aliases(&alias_table, location)
                })
                .log_unwrap();
            for alias in aliases_.unwrap_inner() {
                aliases.push(alias)
            }
        }
        if !delete_locations && aliases.is_empty() {
            warn!("警告：未指定任何有效的别名，将不做任何事情。");
            return;
        }
        if !yes {
            let ans = Self::confirm("再次警告：是否删除？");
            if !ans {
                return;
            }
        }
        database_guard
            .write_once(|w_cxt| {
                if delete_locations {
                    for location in locations {
                        if let Err(e) = LocationTable::delete_location(w_cxt, &location) {
                            if e.is_fatal() {
                                return Err(e);
                            }
                        }
                    }
                }
                let mut alias_table = AliasTable::write(w_cxt)?;
                for alias in aliases {
                    if let Err(e) = AliasTable::delete_alias(&mut alias_table, &alias) {
                        if e.is_fatal() {
                            return Err(e);
                        }
                    }
                }
                Ok(())
            })
            .log_unwrap();
    }
    fn import<'cxt, TypesProtocol, UserProtocol, Cxt>(
        database_guard: &mut DatabaseGuard,
        cxt: Cxt,
        input: Option<PathBuf>,
        course: Option<Course>,
    ) where
        TypesProtocol: TypesProtocolTrait,
        UserProtocol: UserProtocolTrait + std::marker::Send + 'static,
        Cxt: Borrow<<AccountTable<UserProtocol> as TableDefinitionTrait>::Context<'cxt>>,
    {
        let mut do_something = false;
        if let Some(input) = input {
            let contents =
                std::fs::read_to_string(input).expect("文件读取失败，请检查路径是否正确！");
            LocationTable::import_text(database_guard, (), &contents);
            do_something = true;
        }
        if let Some(course_without_info) = course {
            // 获取所有用户。
            let sessions = AccountTable::get_all_sessions(database_guard, cxt).log_unwrap();
            // 获取用户所有的课程。
            let courses = CourseWithInfo::get_from_sessions(sessions.values())
                .ok()
                .unwrap();
            // 找到相应的课程。
            let course = courses
                .into_iter()
                .find(|(course, _)| course.course().eq(&course_without_info))
                .unwrap()
                .0;
            if let Some(session) = sessions.values().next() {
                match course.get_locations::<TypesProtocol>(session) {
                    Ok(locations) => {
                        if locations.is_empty() {
                            warn!("没有从该课程中获取到位置信息。");
                        } else {
                            database_guard
                                .write_once(|w_cxt| {
                                    for (_, l) in locations {
                                        let UnhandledGeoaddr {
                                            unhandled_place_name,
                                            geolocation,
                                        } = l.into_shifted_unhandled_geoaddr();
                                        let _ = LocationTable::insert_location(
                                            w_cxt,
                                            geolocation,
                                            unhandled_place_name,
                                            vec![course.course().clone()],
                                            &[],
                                        );
                                    }
                                    Ok::<_, StoreError>(())
                                })
                                .log_unwrap();
                            do_something = true;
                        }
                    }
                    Err(e) => {
                        warn!("遇到了问题：{e}");
                    }
                }
            }
        }
        if !do_something {
            warn!("未指定任何参数，不做任何事情。")
        }
    }
    fn export<
        'cxt,
        TypesProtocol: TypesProtocolTrait,
        UserProtocol: UserProtocolTrait + std::marker::Send + 'static,
        Cxt: Borrow<<AccountTable<UserProtocol> as TableDefinitionTrait>::Context<'cxt>>,
    >(
        database_guard: &mut DatabaseGuard,
        cxt: Cxt,
        output: Option<PathBuf>,
        course: Option<Course>,
    ) {
        let mut contents = <LocationTable as ImportExportTrait>::export_text(database_guard);
        for content in {
            course
                .and_then(|course_without_info| {
                    // 获取所有用户。
                    let sessions = AccountTable::get_all_sessions(database_guard, cxt).log_unwrap();
                    let courses = CourseWithInfo::get_from_sessions(sessions.values())
                        .unwrap_or_default()
                        .into_keys()
                        .map(|c| (c.course().clone(), c))
                        .collect::<HashMap<_, _>>();
                    courses.get(&course_without_info).and_then(|course| {
                        sessions.values().next().map(|session| {
                            let mut contents = Vec::new();
                            match course.get_locations::<TypesProtocol>(session) {
                                Ok(locations) => {
                                    if locations.is_empty() {
                                        warn!("没有从该课程中获取到位置信息。");
                                    }
                                    for (_, l) in locations {
                                        contents.push(format!(
                                            "{}${}${}\n",
                                            course_without_info,
                                            l.into_shifted_unhandled_geoaddr(),
                                            ""
                                        ));
                                    }
                                }
                                Err(e) => {
                                    warn!("遇到了问题：{e}");
                                }
                            }
                            contents.into_iter()
                        })
                    })
                })
                .into_iter()
                .flatten()
        } {
            contents += content.as_str()
        }
        if contents.is_empty() {
            warn!("没有获取到位置，不做任何事情。")
        } else if let Some(output) = output {
            let _ = std::fs::write(output, contents)
                .map_err(|e| warn!("文件写入出错，请检查路径是否正确！错误信息：{e}"));
        } else {
            println!("{contents}")
        }
    }
    pub fn parse<
        'cxt,
        TypesProtocol: TypesProtocolTrait,
        UserProtocol: UserProtocolTrait + std::marker::Send + 'static,
        Cxt: Borrow<<AccountTable<UserProtocol> as TableDefinitionTrait>::Context<'cxt>>,
    >(
        self,
        database: &Arc<Database>,
        cxt: Cxt,
        app_info: &AppInfo,
    ) {
        let mut g = DatabaseGuard::new(database);
        match self {
            // 添加位置。
            LocationParser::Add {
                location_str,
                alias,
                course,
            } => {
                Self::add(app_info, &mut g, location_str, alias, course);
            }
            LocationParser::Remove { command, yes } => {
                Self::remove(&mut g, command, yes);
            }
            LocationParser::Reduce {
                reduce_type,
                yes,
                all,
                course,
                global,
            } => {
                Self::reduce(&mut g, reduce_type, yes, all, course, global);
            }
            LocationParser::Import { input, course } => {
                Self::import::<TypesProtocol, _, _>(&mut g, cxt, input, course);
            }
            LocationParser::Export { output, course } => {
                Self::export::<TypesProtocol, _, _>(&mut g, cxt, output, course);
            }
        }
    }
}

pub struct LocationCmdApp<
    TypesProtocol = cxlib_internal::protocol::collect::TypesProtocol,
    UserProtocol = cxlib_internal::protocol::collect::UserProtocol,
>(PhantomData<(TypesProtocol, UserProtocol)>);
impl<T, U> Default for LocationCmdApp<T, U> {
    #[inline]
    fn default() -> Self {
        Self(Default::default())
    }
}
impl<
    TypesProtocol: TypesProtocolTrait,
    UserProtocol: UserProtocolTrait + std::marker::Send + 'static,
    Context: AsRef<Arc<Database>> + AsRef<GlobalMultimap<UntypedLoginSolver<UserProtocol>>> + AsRef<AppInfo>,
> AppTrait<Context> for LocationCmdApp<TypesProtocol, UserProtocol>
{
    type OwnedData = LocationParser;
    fn run(&self, cxt: &Context, command: LocationParser) {
        let map: &GlobalMultimap<_> = cxt.as_ref();
        command.parse::<TypesProtocol, UserProtocol, <AccountTable<UserProtocol> as TableDefinitionTrait>::Context<'_>>(cxt.as_ref(), map.clone(), cxt.as_ref())
    }
}
impl<
    'cxt,
    TypesProtocol: TypesProtocolTrait + 'static,
    UserProtocol: UserProtocolTrait + std::marker::Send + 'static,
    Context: AsRef<Arc<Database>>
        + AsRef<GlobalMultimap<UntypedLoginSolver<UserProtocol>>>
        + AsRef<<AccountTable<UserProtocol> as TableDefinitionTrait>::Context<'cxt>>
        + AsRef<AppInfo>
        + 'static,
    OwnedData: 'static,
> CmdMetaAppTrait<Context, OwnedData> for LocationCmdApp<TypesProtocol, UserProtocol>
{
    fn read_owned_data(
        &self,
        _: &Context,
        matches: &[&ArgMatches],
    ) -> <Self as AppTrait<Context, ()>>::OwnedData {
        let matches = matches.last().unwrap();
        LocationParser::from_arg_matches(matches).unwrap()
    }
}
