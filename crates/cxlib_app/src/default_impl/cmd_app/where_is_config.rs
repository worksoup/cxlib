use crate::{AppTrait, CmdMetaAppTrait};
use clap::{ArgMatches, Parser};
#[derive(Debug, Parser, Clone)]
// TODO: build.rs 中通过环境变量设置 alias.
#[command(name = "where-is-config", alias = "w")]
/// 显示配置文件夹位置。
pub struct WhereIsConfigParser;
pub struct WhereIsConfigCmdApp;
impl<Context> AppTrait<Context> for WhereIsConfigCmdApp {
    type OwnedData = WhereIsConfigParser;

    fn run(&self, _data: &Context, _: WhereIsConfigParser) {
        println!(
            "{}",
            &crate::cxlib::store::Dir::get_config_dir()
                .into_os_string()
                .to_string_lossy()
                .to_string()
        );
    }
}
impl<Context: 'static, OwnedData: 'static> CmdMetaAppTrait<Context, OwnedData>
    for WhereIsConfigCmdApp
{
    fn read_owned_data(
        &self,
        _: &Context,
        _: &[&ArgMatches],
    ) -> <Self as AppTrait<Context, ()>>::OwnedData {
        WhereIsConfigParser
    }
}
