use std::marker::PhantomData;

use crate::{AppTrait, CmdMetaAppTrait};
use clap::{ArgMatches, Parser};
use cxlib_store::DirTrait;
#[derive(Debug, Parser, Clone)]
// TODO: build.rs 中通过环境变量设置 alias.
#[command(name = "where-is-config", alias = "w")]
/// 显示配置文件夹位置。
pub struct WhereIsConfigParser;
pub struct WhereIsConfigCmdApp<D = cxlib_store::Dir>(PhantomData<D>);
impl<D> Default for WhereIsConfigCmdApp<D> {
    #[inline]
    fn default() -> Self {
        Self(Default::default())
    }
}
impl<D: DirTrait, Context: AsRef<D>> AppTrait<Context> for WhereIsConfigCmdApp<D> {
    type OwnedData = WhereIsConfigParser;

    #[inline]
    fn run(&self, cxt: &Context, _: WhereIsConfigParser) {
        println!(
            "{}",
            &cxt.as_ref()
                .get_config_dir()
                .into_os_string()
                .to_string_lossy()
                .to_string()
        );
    }
}
impl<D: DirTrait + 'static, Context: 'static + std::convert::AsRef<D>, OwnedData: 'static>
    CmdMetaAppTrait<Context, OwnedData> for WhereIsConfigCmdApp<D>
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
