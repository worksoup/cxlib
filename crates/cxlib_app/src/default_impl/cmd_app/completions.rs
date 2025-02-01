use crate::{AppTrait, CmdApp, CmdMetaAppTrait};
use clap::{arg, ArgMatches, Command, CommandFactory, FromArgMatches, Parser};
use clap_complete_command::Shell;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(name = "completions")]
#[clap(about = "生成命令补全文件。")]
pub struct CompletionsParser {
    /// 补全的 Shell 类型。
    #[arg(value_enum)]
    shell: Shell,
    #[arg(short, long)]
    output: Option<PathBuf>,
}

pub struct CompletionsCmdApp {
    command: Command,
}
impl CompletionsCmdApp {
    pub fn new() -> Self {
        let command = CompletionsParser::command();
        Self { command }
    }
}
impl Default for CompletionsCmdApp {
    fn default() -> Self {
        Self::new()
    }
}
impl<Context: AsRef<Command>> AppTrait<Context> for CompletionsCmdApp {
    type OwnedData = CompletionsParser;

    fn run(&self, command: &Context, CompletionsParser { shell, output }: Self::OwnedData) {
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
impl<Context: AsRef<Command> + 'static> CmdMetaAppTrait<CmdApp<Context>, Context>
    for CompletionsCmdApp
{
    fn subcommand(&self) -> Option<&Command> {
        Some(&self.command)
    }

    fn read_owned_data(
        &self,
        _: &Context,
        matches: &[&ArgMatches],
    ) -> <Self as AppTrait<Context, ()>>::OwnedData {
        CompletionsParser::from_arg_matches(matches.last().unwrap()).unwrap()
    }
}
