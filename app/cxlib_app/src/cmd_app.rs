use crate::{AppTrait, MetaAppTrait};
use clap::{ArgMatches, Args, Command, CommandFactory, Subcommand};
use std::collections::HashMap;

pub trait CmdMetaAppTrait<Context: 'static = (), OwnedData: 'static = (), Output: 'static = ()>
where
    Self: MetaAppTrait<CmdApp<Context, OwnedData, Output>, Context, Output>,
{
    fn read_owned_data(
        &self,
        context: &Context,
        matches: &[&ArgMatches],
    ) -> <Self as AppTrait<Context, Output>>::OwnedData;
}
type MetaAppInvoker<Context, Output> = Box<dyn Fn(&Context, &Vec<&ArgMatches>) -> Output>;
impl<Context, OwnedData, Output, T>
    MetaAppTrait<CmdApp<Context, OwnedData, Output>, Context, Output> for T
where
    Context: 'static,
    OwnedData: 'static,
    Output: 'static,
    T: CmdMetaAppTrait<Context, OwnedData, Output> + 'static,
    <Self as AppTrait<Context, Output>>::OwnedData: CommandFactory,
{
    fn register(self, app: CmdApp<Context, OwnedData, Output>) -> CmdApp<Context, OwnedData, Output>
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

impl<Context, OwnedData, Output> AppTrait<Context, Output> for CmdApp<Context, OwnedData, Output> {
    type OwnedData = OwnedData;
    fn run(&self, data: &Context, owned_data: OwnedData) -> Output {
        let command = self.command.clone();
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

impl<Context, OwnedData, Output> CmdMetaAppTrait<Context, OwnedData, Output>
    for CmdApp<Context, OwnedData, Output>
where
    Context: 'static,
    OwnedData: CommandFactory + 'static,
    Output: 'static,
{
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
    fn insert_meta_app<MetaApp: CmdMetaAppTrait<Context, OwnedData, Output> + 'static>(
        mut self,
        meta_app: MetaApp,
    ) -> Self
    where
        <MetaApp as AppTrait<Context, Output>>::OwnedData: CommandFactory,
        Context: 'static,
        OwnedData: 'static,
        Output: 'static,
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

    fn get_meta_app(&self, ident: &str) -> Option<&MetaAppInvoker<Context, Output>> {
        self.meta_app_invokers.get(ident)
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

#[cfg(test)]
mod tests {
    use crate::AccountParser;
    use clap::CommandFactory;

    #[test]
    fn test() {
        let command = AccountParser::command();
        let matches = command.get_matches_from(["a", "+", "145"]);
        let subcommand = matches.subcommand();
        println!("{subcommand:?}");
    }
}
