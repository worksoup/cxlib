use crate::{AppTrait, CmdApp, CmdMetaAppTrait};
use clap::{arg, ArgMatches, Args, Command, CommandFactory, FromArgMatches, Parser};
use clap_complete_command::Shell;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
// TODO: build.rs 中通过环境变量设置 alias.
#[command(name = "completion", alias = "c")]
/// 生成命令补全文件。
pub struct CompletionParser {
    /// 补全的 Shell 类型。
    #[arg(value_enum)]
    shell: Shell,
    #[arg(short, long)]
    output: Option<PathBuf>,
}

pub struct CompletionCmdApp;
impl<Context: AsRef<Command>> AppTrait<Context> for CompletionCmdApp {
    type OwnedData = CompletionParser;

    fn run(&self, command: &Context, CompletionParser { shell, output }: Self::OwnedData) {
        let command: &Command = command.as_ref();
        let mut command = command.clone();
        if let Some(output) = output {
            shell
                .generate_to(&mut command, output)
                .map_err(|e| log::warn!("文件写入出错，请检查路径是否正确！错误信息：{e}"))
                .unwrap();
        } else {
            shell.generate(&mut command, &mut std::io::stdout());
        }
    }
}
impl<Context: AsRef<Command> + 'static, OwnedData: 'static> CmdMetaAppTrait<Context, OwnedData>
    for CompletionCmdApp
{
    fn read_owned_data(
        &self,
        _: &Context,
        matches: &[&ArgMatches],
    ) -> <Self as AppTrait<Context, ()>>::OwnedData {
        CompletionParser::from_arg_matches(matches.last().unwrap()).unwrap()
    }
}
