mod account;
mod accounts;
mod activity;
mod course;
mod list;
mod location;
mod locations;
mod where_is_config;

pub use account::*;
pub use accounts::*;
pub use activity::*;
pub use course::*;
pub use list::*;
pub use location::*;
pub use locations::*;
pub use where_is_config::*;

#[cfg(feature = "completion")]
mod completions;
#[cfg(feature = "completion")]
pub use completions::*;

use clap::Command;
use cxlib_internal::default_impl::store::DataBase;
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
