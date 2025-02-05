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
pub use location::*;
pub use locations::*;
pub use where_is_config::*;

#[cfg(feature = "completion")]
mod completions;
#[cfg(feature = "completion")]
pub use completions::*;

use clap::Command;
use cxlib_internal::{
    captcha::utils::get_now_timestamp_mills,
    default_impl::store::{CourseData, DataBase},
};
use std::cmp;
pub struct CmdAppContext {
    db: DataBase,
    command: Command,
}
impl CmdAppContext {
    pub fn new(db: DataBase, command: Command) -> Self {
        Self { db, command }
    }
}
impl AsRef<Command> for CmdAppContext {
    fn as_ref(&self) -> &Command {
        &self.command
    }
}
impl AsRef<DataBase> for CmdAppContext {
    fn as_ref(&self) -> &DataBase {
        &self.db
    }
}

pub trait CourseDataFilterAndSorterTrait {
    fn filter(a: &CourseData) -> bool;
    fn sorter(a: &CourseData, b: &CourseData) -> cmp::Ordering;
}
pub struct DefaultCourseDataSorter;
impl CourseDataFilterAndSorterTrait for DefaultCourseDataSorter {
    fn filter(a: &CourseData) -> bool {
        !a.class_ended()
            && (*a.recently_used_timestamp() == u64::MAX || {
                let now = (get_now_timestamp_mills() / 1000) as u64;
                now - a.recently_used_timestamp() < 160 * 24 * 60 * 60
            })
    }
    fn sorter(a: &CourseData, b: &CourseData) -> cmp::Ordering {
        let a = a.recently_used_timestamp();
        let b = b.recently_used_timestamp();
        let now = (get_now_timestamp_mills() / 1000) as u64;
        let da = (now - a) / (24 * 60 * 60);
        let db = (now - b) / (24 * 60 * 60);
        let da = da == 7;
        let db = db == 7;
        if da == db {
            b.cmp(a)
        } else if da {
            cmp::Ordering::Greater
        } else {
            cmp::Ordering::Less
        }
    }
}
