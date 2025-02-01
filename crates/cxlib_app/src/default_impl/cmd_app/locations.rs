use crate::{AppTrait, CmdApp, CmdMetaAppTrait};
use clap::{arg, ArgMatches, Command, CommandFactory, FromArgMatches, Parser};
use cxlib_internal::default_impl::store::{AliasTable, DataBase, LocationTable};

#[derive(Parser, Debug, Clone)]
#[command(name = "locations")]
#[clap(about = "列出所有位置。")]
pub struct LocationsParser {
    /// 列出全局位置。
    #[arg(short, long)]
    global: bool,
    /// 列出指定课程的位置。
    #[arg(short, long)]
    course: Option<i64>,
    /// 以更好的格式显示结果。
    #[arg(short, long)]
    pretty: bool,
    /// 精简显示结果。
    #[arg(short, long)]
    short: bool,
}

pub struct LocationsCmdApp {
    command: Command,
}
impl Default for LocationsCmdApp {
    fn default() -> Self {
        Self::new()
    }
}

impl LocationsCmdApp {
    pub fn new() -> LocationsCmdApp {
        let command = LocationsParser::command();
        LocationsCmdApp { command }
    }
}

impl<Context: AsRef<DataBase>> AppTrait<Context> for LocationsCmdApp {
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
        let db = context.as_ref();
        let course_id = course.or(if global { Some(-1) } else { None });
        if short {
            let locations = if let Some(course_id) = course_id {
                LocationTable::get_location_map_by_course(db, course_id)
            } else {
                LocationTable::get_locations(db)
                    .into_iter()
                    .map(|(k, v)| (k, v.1))
                    .collect()
            };
            for (_, location) in locations {
                println!("{}", location,)
            }
        } else if let Some(course_id) = course_id {
            // 列出指定课程的位置。
            let locations = LocationTable::get_location_map_by_course(db, course_id);
            if pretty {
                for (location_id, location) in locations {
                    println!(
                        "位置id: {}, 位置: {},\n\t别名: {:?}",
                        location_id,
                        location,
                        AliasTable::get_aliases(db, location_id)
                    )
                }
            } else {
                for (location_id, location) in locations {
                    println!(
                        "{}${}${:?}",
                        location_id,
                        location,
                        AliasTable::get_aliases(db, location_id)
                    )
                }
            }
        } else {
            // 列出所有位置。
            let locations = LocationTable::get_locations(db);
            if pretty {
                for (location_id, (course_id, location)) in locations {
                    println!(
                        "位置id: {}, 课程号: {}, 位置: {},\n\t别名: {:?}",
                        location_id,
                        course_id,
                        location,
                        AliasTable::get_aliases(db, location_id)
                    )
                }
            } else {
                for (location_id, (course_id, location)) in locations {
                    println!(
                        "{}${}${}${:?}",
                        location_id,
                        course_id,
                        location,
                        AliasTable::get_aliases(db, location_id)
                    )
                }
            }
        }
    }
}
impl<Context: AsRef<DataBase> + 'static> CmdMetaAppTrait<CmdApp<Context>, Context>
    for LocationsCmdApp
{
    fn subcommand(&self) -> Option<&Command> {
        Some(&self.command)
    }

    fn read_owned_data(&self, _: &Context, matches: &[&ArgMatches]) -> Self::OwnedData {
        let matches = matches.last().unwrap();
        LocationsParser::from_arg_matches(matches).unwrap()
    }
}
