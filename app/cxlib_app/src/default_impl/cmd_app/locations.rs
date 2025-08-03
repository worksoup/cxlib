use std::sync::Arc;

use crate::{
    AliasTable, AppTrait, CmdMetaAppTrait, CourseTable, LocationTable, NormalTableTrait,
    database_guard::DatabaseGuard,
};
use clap::{ArgMatches, FromArgMatches, Parser, arg};
use cxlib_error_utils::CxlibResultUtils;
use cxlib_internal::types::Course;
use redb::Database;

#[derive(Parser, Debug, Clone)]
// TODO: build.rs 中通过环境变量设置 alias.
#[command(name = "locations", alias = "lsl")]
/// 列出所有位置。
pub struct LocationsParser {
    /// 列出全局位置。
    #[arg(short, long)]
    global: bool,
    /// 列出指定课程的位置。
    #[arg(short, long)]
    course: Option<Course>,
    /// 以更好的格式显示结果。
    #[arg(short, long)]
    pretty: bool,
    /// 精简显示结果。
    #[arg(short, long)]
    short: bool,
}
#[derive(Default)]
pub struct LocationsCmdApp;

impl<Context: AsRef<Arc<Database>>> AppTrait<Context> for LocationsCmdApp {
    type OwnedData = LocationsParser;

    fn run(
        &self,
        context: &Context,
        LocationsParser {
            global,
            course,
            pretty,
            short,
        }: Self::OwnedData,
    ) {
        let g = DatabaseGuard::new(context.as_ref());
        let course = course.or(if global {
            Some(Course::global_course())
        } else {
            None
        });
        if short {
            if let Some(course) = course {
                let binding = g
                    .read(|r_cxt| {
                        let course_table = CourseTable::read(r_cxt)?;
                        CourseTable::get_course(&course_table, &course)
                    })
                    .log_unwrap()
                    .unwrap_inner();
                let locations = binding.iter().flat_map(|(_, data)| data.locations());
                for location in locations {
                    println!("{location}")
                }
            } else {
                let geoaddrs = g
                    .read(|r_cxt| {
                        let location_table = LocationTable::read(r_cxt)?;
                        LocationTable::get_geoaddrs(&location_table)
                    })
                    .log_unwrap()
                    .unwrap_inner()
                    .into_keys();
                for geoaddr in geoaddrs {
                    println!("{geoaddr}")
                }
            };
        } else if let Some(course) = course {
            // 列出指定课程的位置。
            let locations = g
                .read(|r_cxt| {
                    let course_table = CourseTable::read(r_cxt)?;
                    CourseTable::get_course(&course_table, &course)
                })
                .log_unwrap()
                .unwrap_inner();
            let aliases = g
                .read(|r_cxt| {
                    let alias_table = AliasTable::read(r_cxt)?;
                    AliasTable::get_all_aliases(&alias_table)
                })
                .log_unwrap()
                .unwrap_inner();
            if let Some((_, data)) = locations {
                for location in data.locations() {
                    let aliases = aliases
                        .iter()
                        .filter_map(|(a, l)| {
                            if location.eq(l) {
                                Some(a.as_str())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    if pretty {
                        println!("课程：{course}, 位置: {location},\n\t别名: {aliases:?}")
                    } else {
                        println!("{course}${location}${aliases:?}")
                    }
                }
            }
        } else {
            // 列出所有位置。
            let locations = g
                .read(|r_cxt| {
                    let course_table = LocationTable::read(r_cxt)?;
                    LocationTable::get_geoaddrs(&course_table)
                })
                .log_unwrap()
                .unwrap_inner();
            let aliases = g
                .read(|r_cxt| {
                    let alias_table = AliasTable::read(r_cxt)?;
                    AliasTable::get_all_aliases(&alias_table)
                })
                .log_unwrap()
                .unwrap_inner();
            for (location, courses) in locations {
                let aliases = aliases
                    .iter()
                    .filter_map(|(a, l)| {
                        if location.eq(l) {
                            Some(a.as_str())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                let mut courses = courses.iter();
                let first = courses.next();

                let courses_str = if let Some(first) = first {
                    let courses = courses.map(ToString::to_string);
                    courses.fold(first.to_string(), |a, b| a + "/" + &b)
                } else {
                    String::new()
                };
                if pretty {
                    println!("课程：{courses_str}, 位置: {location},\n\t别名: {aliases:?}")
                } else {
                    println!("{courses_str}${location}${aliases:?}")
                }
            }
        }
    }
}
impl<Context: AsRef<Arc<Database>> + 'static, OwnedData: 'static>
    CmdMetaAppTrait<Context, OwnedData> for LocationsCmdApp
{
    #[inline]
    fn read_owned_data(&self, _: &Context, matches: &[&ArgMatches]) -> Self::OwnedData {
        let matches = matches.last().unwrap();
        LocationsParser::from_arg_matches(matches).unwrap()
    }
}
