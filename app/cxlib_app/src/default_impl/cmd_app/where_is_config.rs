use clap::{ArgMatches, Parser};
use cxlib_store::ConfigDir;

use crate::{AppTrait, CmdMetaAppTrait};
#[derive(Debug, Parser, Clone)]
// TODO: build.rs 中通过环境变量设置 alias.
#[command(name = "where-is-config", alias = "w")]
/// 显示配置文件夹位置。
pub struct WhereIsConfigParser;
pub struct WhereIsConfigCmdApp {}
impl Default for WhereIsConfigCmdApp {
    #[inline]
    fn default() -> Self {
        Self {}
    }
}
impl<Context: AsRef<ConfigDir>> AppTrait<Context> for WhereIsConfigCmdApp {
    type OwnedData = WhereIsConfigParser;

    #[inline]
    fn run(&self, cxt: &Context, _: WhereIsConfigParser) {
        println!(
            "{}",
            &cxt.as_ref()
                .get_config_dir()
                .as_os_str()
                .to_string_lossy()
                .to_string()
        );
    }
}
impl<Context: 'static + AsRef<ConfigDir>, OwnedData: 'static> CmdMetaAppTrait<Context, OwnedData>
    for WhereIsConfigCmdApp
{
    #[inline]
    fn read_owned_data(
        &self,
        _: &Context,
        _: &[&ArgMatches],
    ) -> <Self as AppTrait<Context, ()>>::OwnedData {
        WhereIsConfigParser
    }
}
