use crate::{AppTrait, MetaAppTrait};
use clap::{ArgMatches, Args, Command, CommandFactory, Subcommand};
use std::collections::HashMap;

/// 该文档为AI生成。命令行元应用trait，扩展了MetaAppTrait的功能
///
/// 提供从命令行参数解析自有数据的能力，用于构建复杂的命令行应用结构
pub trait CmdMetaAppTrait<Context = (), OwnedData = (), Output = ()>
where
    Self: MetaAppTrait<CmdApp<Context, OwnedData, Output>, Context, Output>,
{
    /// 该文档为AI生成。从命令行匹配结果中读取自有数据
    ///
    /// # 参数
    /// - `context`: 应用上下文
    /// - `matches`: 命令行参数匹配结果链
    fn read_owned_data(
        &self,
        context: &Context,
        matches: &[&ArgMatches],
    ) -> <Self as AppTrait<Context, Output>>::OwnedData;
}

/// 该文档为AI生成。元应用调用器类型，用于执行注册的元应用
type MetaAppInvoker<Context, Output> = Box<dyn Fn(&Context, &Vec<&ArgMatches>) -> Output>;

/// 该文档为AI生成。为实现了CmdMetaAppTrait的类型自动实现MetaAppTrait
impl<Context, OwnedData, Output, T>
    MetaAppTrait<CmdApp<Context, OwnedData, Output>, Context, Output> for T
where
    T: CmdMetaAppTrait<Context, OwnedData, Output> + 'static,
    <Self as AppTrait<Context, Output>>::OwnedData: CommandFactory,
{
    /// 该文档为AI生成。将元应用注册到命令行应用结构中
    fn register(self, app: CmdApp<Context, OwnedData, Output>) -> CmdApp<Context, OwnedData, Output>
    where
        Self: Sized,
    {
        app.insert_meta_app(self)
    }
}

/// 该文档为AI生成。主应用调用器类型，用于执行主应用逻辑
type MainAppInvoker<Context, OwnedData, Output> =
    Box<dyn Fn(&CmdApp<Context, OwnedData, Output>, &Context, OwnedData, &ArgMatches) -> Output>;

/// 该文档为AI生成。自有数据读取器类型，用于从命令行参数解析自有数据
type OwnedDataReader<Context, OwnedData> = Box<dyn Fn(&Context, &[&ArgMatches]) -> OwnedData>;

/// 该文档为AI生成。命令行应用核心结构
///
/// 封装了clap的Command功能，支持：
/// - 主应用逻辑执行
/// - 元应用（子命令）注册和执行
/// - 自有数据解析
pub struct CmdApp<Context = (), OwnedData = (), Output = ()> {
    /// 该文档为AI生成。clap命令结构
    command: Command,
    /// 该文档为AI生成。主应用逻辑执行器
    app: MainAppInvoker<Context, OwnedData, Output>,
    /// 该文档为AI生成。自有数据读取器
    read_owned_data: OwnedDataReader<Context, OwnedData>,
    /// 该文档为AI生成。注册的元应用调用器映射表
    meta_app_invokers: HashMap<String, MetaAppInvoker<Context, Output>>,
}

/// 该文档为AI生成。为CmdApp实现AppTrait
impl<Context, OwnedData, Output> AppTrait<Context, Output> for CmdApp<Context, OwnedData, Output> {
    type OwnedData = OwnedData;

    /// 该文档为AI生成。运行应用逻辑
    ///
    /// 根据命令行参数决定执行主应用或元应用
    fn run(&self, data: &Context, owned_data: OwnedData) -> Output {
        let command = self.command.clone();
        let matches = command.get_matches();
        let mut matches_vec = vec![&matches];

        if let Some((command_name, sub_matches)) = matches.subcommand() {
            matches_vec.push(sub_matches);
            self.get_meta_app(command_name).expect("元应用未注册")(data, &matches_vec)
        } else {
            (self.app)(self, data, owned_data, &matches)
        }
    }
}

/// 该文档为AI生成。为CmdApp实现CmdMetaAppTrait
impl<Context, OwnedData, Output> CmdMetaAppTrait<Context, OwnedData, Output>
    for CmdApp<Context, OwnedData, Output>
where
    Context: 'static,
    OwnedData: CommandFactory + 'static,
    Output: 'static,
{
    /// 该文档为AI生成。从命令行参数解析自有数据
    fn read_owned_data(
        &self,
        context: &Context,
        matches: &[&ArgMatches],
    ) -> <Self as AppTrait<Context, Output>>::OwnedData {
        (self.read_owned_data)(context, matches)
    }
}

/// 该文档为AI生成。CmdApp的实现方法
impl<Context, OwnedData, Output> CmdApp<Context, OwnedData, Output> {
    /// 该文档为AI生成。创建新的命令行应用实例
    ///
    /// # 注意
    /// 使用此方法创建的应用需要后续配置主应用和自有数据读取器
    pub fn new(command: Command) -> Self {
        Self {
            command,
            app: Box::new(|_, _, _, _| unimplemented!("主应用逻辑未配置")),
            read_owned_data: Box::new(|_, _| unimplemented!("自有数据读取器未配置")),
            meta_app_invokers: Default::default(),
        }
    }

    /// 该文档为AI生成。完整构建命令行应用实例
    ///
    /// # 参数
    /// - `command`: clap命令结构
    /// - `main_logic`: 主应用执行逻辑
    /// - `read_owned_data`: 自有数据读取逻辑
    pub fn build(
        command: Command,
        main_logic: impl Fn(&Self, &Context, OwnedData, &ArgMatches) -> Output + 'static,
        read_owned_data: impl Fn(&Context, &[&ArgMatches]) -> OwnedData + 'static,
    ) -> Self {
        let app = Box::new(main_logic);
        let read_owned_data = Box::new(read_owned_data);
        Self {
            command,
            app,
            read_owned_data,
            meta_app_invokers: Default::default(),
        }
    }

    /// 该文档为AI生成。内部方法：插入主应用调用器
    fn insert_main_app_invoker<C: CmdMetaAppTrait<Context, OwnedData, Output> + 'static>(
        mut self,
        main_app: C,
    ) -> Self
    where
        OwnedData: 'static,
        Output: 'static,
        Context: 'static,
    {
        let main_app_invoker =
            move |_self_: &Self, context: &Context, _: OwnedData, args: &ArgMatches| {
                let data = main_app.read_owned_data(context, &[args]);
                main_app.run(context, data)
            };
        self.app = Box::new(main_app_invoker);
        self
    }

    /// 该文档为AI生成。注册使用参数的主应用
    ///
    /// # 说明
    /// 主应用的自有数据必须实现clap::Args
    pub fn main_app<C: CmdMetaAppTrait<Context, OwnedData, Output> + 'static>(
        mut self,
        main_app: C,
    ) -> Self
    where
        OwnedData: 'static,
        Output: 'static,
        Context: 'static,
        <C as AppTrait<Context, Output>>::OwnedData: clap::Args,
    {
        self.command = <C as AppTrait<Context, Output>>::OwnedData::augment_args(self.command);
        self.insert_main_app_invoker(main_app)
    }

    /// 该文档为AI生成。注册使用子命令的主应用
    ///
    /// # 说明
    /// 主应用的自有数据必须实现clap::Subcommand
    pub fn main_app_with_subcommand<C: CmdMetaAppTrait<Context, OwnedData, Output> + 'static>(
        mut self,
        main_app: C,
    ) -> Self
    where
        OwnedData: 'static,
        Output: 'static,
        Context: 'static,
        <C as AppTrait<Context, Output>>::OwnedData: clap::Subcommand,
    {
        self.command =
            <C as AppTrait<Context, Output>>::OwnedData::augment_subcommands(self.command);
        self.insert_main_app_invoker(main_app)
    }

    /// 该文档为AI生成。内部方法：插入元应用
    fn insert_meta_app<MetaApp: CmdMetaAppTrait<Context, OwnedData, Output> + 'static>(
        mut self,
        meta_app: MetaApp,
    ) -> Self
    where
        <MetaApp as AppTrait<Context, Output>>::OwnedData: CommandFactory,
    {
        let command = <MetaApp as AppTrait<Context, Output>>::OwnedData::command();
        let name = command.get_name().to_owned();
        self.command = self.command.subcommand(command);

        let run_meta_app = move |data: &Context, matches: &Vec<&ArgMatches>| -> Output {
            let owned_data = meta_app.read_owned_data(data, matches);
            meta_app.run(data, owned_data)
        };

        self.meta_app_invokers.insert(name, Box::new(run_meta_app));
        self
    }

    /// 该文档为AI生成。获取已注册的元应用
    fn get_meta_app(&self, ident: &str) -> Option<&MetaAppInvoker<Context, Output>> {
        self.meta_app_invokers.get(ident)
    }

    /// 该文档为AI生成。配置自有数据读取器
    pub fn owned_data_builder(
        mut self,
        read_owned_data: impl Fn(&Context, &[&ArgMatches]) -> OwnedData + 'static,
    ) -> Self {
        self.read_owned_data = Box::new(read_owned_data);
        self
    }

    /// 该文档为AI生成。初始化并运行应用
    ///
    /// # 参数
    /// - `init`: 初始化函数，返回上下文和自有数据
    pub fn init_and_run(&self, init: impl FnOnce(&Self) -> (Context, OwnedData)) -> Output
    where
        OwnedData: 'static,
        Output: 'static,
        Context: 'static,
    {
        let (context, data) = init(self);
        self.run(&context, data)
    }

    /// 该文档为AI生成。获取内部clap命令的引用
    pub fn command(&self) -> &Command {
        &self.command
    }
}

#[cfg(test)]
mod tests {
    use crate::AccountParser;
    use clap::CommandFactory;

    #[test]
    fn test() {
        let command = <AccountParser>::command();
        let matches = command.get_matches_from(["a", "+", "145"]);
        let subcommand = matches.subcommand();
        println!("{subcommand:?}");
    }
}
