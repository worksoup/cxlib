use crate::{AppTrait, CmdApp, CmdMetaAppTrait};
use clap::{ArgMatches, Command, CommandFactory, Parser};
#[derive(Debug, Parser, Clone)]
#[command(name = "where-is-config")]
#[clap(about = "显示配置文件夹位置。")]
pub struct WhereIsConfigParser;
pub struct WhereIsConfigCmdApp {
    command: Command,
}
impl WhereIsConfigCmdApp {
    pub fn new() -> WhereIsConfigCmdApp {
        let command = WhereIsConfigParser::command();
        WhereIsConfigCmdApp { command }
    }
}
impl Default for WhereIsConfigCmdApp {
    fn default() -> WhereIsConfigCmdApp {
        WhereIsConfigCmdApp::new()
    }
}
impl<Context> AppTrait<Context> for WhereIsConfigCmdApp {
    type OwnedData = ();

    fn run(&self, _data: &Context, _: ()) {
        println!(
            "{}",
            &crate::cxlib::store::Dir::get_config_dir()
                .into_os_string()
                .to_string_lossy()
                .to_string()
        );
    }
}
impl<Context: 'static> CmdMetaAppTrait<CmdApp<Context>, Context> for WhereIsConfigCmdApp {
    fn subcommand(&self) -> Option<&Command> {
        Some(&self.command)
    }

    fn read_owned_data(
        &self,
        _: &Context,
        _: &[&ArgMatches],
    ) -> <Self as AppTrait<Context, ()>>::OwnedData {
    }
}
