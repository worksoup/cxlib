use crate::{AppTrait, MetaAppTrait};
use clap::{Arg, ArgMatches, Command};
use std::collections::HashMap;

pub trait CmdMetaAppTrait<App: CmdAppTrait<Context, Output>, Context = (), Output = ()>:
    MetaAppTrait<App, Context, Output>
{
    fn args(&self) -> Vec<Arg> {
        vec![]
    }
    fn subcommand(&self) -> Option<&Command> {
        None
    }
    fn push_subcommand(&self, mut cmd: Command) -> Command {
        if let Some(subcommand) = self.subcommand() {
            cmd = cmd.subcommand(subcommand);
        }
        cmd
    }
    fn read_owned_data(
        &self,
        context: &Context,
        matches: &[&ArgMatches],
    ) -> <Self as AppTrait<Context, Output>>::OwnedData;
}
type MetaAppInvoker<Context, Output> = Box<dyn Fn(&Context, &Vec<&ArgMatches>) -> Output>;
pub trait CmdAppTrait<Context = (), Output = ()>: AppTrait<Context, Output> {
    fn insert_meta_app<MetaApp: CmdMetaAppTrait<Self, Context, Output> + 'static>(
        self,
        meta_app: MetaApp,
    ) -> Self
    where
        Self: Sized;
    fn get_meta_app(&self, ident: &str) -> Option<&MetaAppInvoker<Context, Output>>;
}
impl<
        App: CmdAppTrait<Context, Output>,
        Context,
        Output,
        T: CmdMetaAppTrait<App, Context, Output> + 'static,
    > MetaAppTrait<App, Context, Output> for T
{
    fn register(self, app: App) -> App
    where
        Self: Sized,
    {
        app.insert_meta_app(self)
    }
}
type MainAppInvoker<Context, OwnedData, Output> =
    Box<dyn Fn(&CmdApp<Context, OwnedData, Output>, &Context, OwnedData, &ArgMatches) -> Output>;
type OwnedDataReader<Context, OwnedData> = Box<dyn Fn(&Context, &[&ArgMatches]) -> OwnedData>;
pub struct CmdApp<Context = (), OwnedData = (), Output = ()> {
    command: Command,
    app: MainAppInvoker<Context, OwnedData, Output>,
    read_owned_data: OwnedDataReader<Context, OwnedData>,
    meta_app_invokers: HashMap<String, MetaAppInvoker<Context, Output>>,
}

impl<Context: 'static, OwnedData: 'static, Output: 'static> AppTrait<Context, Output>
    for CmdApp<Context, OwnedData, Output>
{
    type OwnedData = OwnedData;
    fn run(&self, data: &Context, owned_data: OwnedData) -> Output {
        let command = <CmdApp<Context, OwnedData, Output> as CmdMetaAppTrait<
            Self,
            Context,
            Output,
        >>::subcommand(self)
        .cloned()
        .unwrap();
        let matches = command.get_matches();
        let mut matches_vec = vec![&matches];
        let subcommand = matches.subcommand();
        if let Some((command_name, matches)) = subcommand {
            matches_vec.push(matches);
            let app = self.get_meta_app(command_name).unwrap();
            app(data, &matches_vec)
        } else {
            (self.app)(self, data, owned_data, &matches)
        }
    }
}

impl<Context: 'static, OwnedData: 'static, Output: 'static> CmdAppTrait<Context, Output>
    for CmdApp<Context, OwnedData, Output>
{
    fn insert_meta_app<MetaApp: CmdMetaAppTrait<Self, Context, Output> + 'static>(
        mut self,
        meta_app: MetaApp,
    ) -> Self {
        let command = meta_app.subcommand();
        let args = meta_app.args();
        for arg in args {
            self.command = self.command.arg(arg);
        }
        if let Some(command) = command {
            self.command = self.command.subcommand(command);
            let key = command.get_name().to_owned();
            let run_meta_app = move |data: &Context, matches: &Vec<&ArgMatches>| -> Output {
                let owned_data = meta_app.read_owned_data(data, matches);
                meta_app.run(data, owned_data)
            };
            self.meta_app_invokers.insert(key, Box::new(run_meta_app));
        }
        self
    }

    fn get_meta_app(&self, ident: &str) -> Option<&MetaAppInvoker<Context, Output>> {
        self.meta_app_invokers.get(ident)
    }
}
impl<App: CmdAppTrait<Context, Output>, Context: 'static, OwnedData: 'static, Output: 'static>
    CmdMetaAppTrait<App, Context, Output> for CmdApp<Context, OwnedData, Output>
{
    fn args(&self) -> Vec<Arg> {
        self.command.get_arguments().cloned().collect()
    }
    fn subcommand(&self) -> Option<&Command> {
        Some(&self.command)
    }
    fn read_owned_data(
        &self,
        context: &Context,
        matches: &[&ArgMatches],
    ) -> <Self as AppTrait<Context, Output>>::OwnedData {
        (self.read_owned_data)(context, matches)
    }
}
impl<Context, OwnedData, Output> CmdApp<Context, OwnedData, Output> {
    pub fn new(command: Command) -> Self {
        Self {
            command,
            app: Box::new(|_, _, _, _| unimplemented!()),
            read_owned_data: Box::new(|_, _| unimplemented!()),
            meta_app_invokers: Default::default(),
        }
    }
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
    pub fn main_cmd_app(
        mut self,
        main_app: impl CmdMetaAppTrait<Self, Context, Output> + 'static,
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
    pub fn owned_data_builder(
        mut self,
        read_owned_data: impl Fn(&Context, &[&ArgMatches]) -> OwnedData + 'static,
    ) -> Self {
        self.read_owned_data = Box::new(read_owned_data);
        self
    }
    pub fn init_and_run(&self, init: impl FnOnce(&Self) -> (Context, OwnedData)) -> Output
    where
        OwnedData: 'static,
        Output: 'static,
        Context: 'static,
    {
        let (context, data) = init(self);
        self.run(&context, data)
    }
    pub fn command(&self) -> &Command {
        &self.command
    }
}
