mod cmd_app;
mod default_impl;
mod error;
mod global_multimap;
mod login_solver;

pub use cmd_app::*;
pub use cxlib_store::*;
pub use default_impl::*;
pub use error::*;
pub use global_multimap::*;
pub use login_solver::*;

pub trait MetaAppTrait<App: AppTrait<Context, Output>, Context = (), Output = ()>:
    AppTrait<Context, Output>
{
    fn register(self, app: App) -> App
    where
        Self: Sized;
}
pub trait ConstructFromTrait<App: AppTrait<Context, Output>, Context = (), Output = ()>:
    AppTrait<Context, Output>
{
    fn construct_from(app: &App) -> Self;
}
impl<A: AppTrait<C, O>, C, O, T: AppTrait<C, O> + Default> ConstructFromTrait<A, C, O> for T {
    fn construct_from(_: &A) -> Self {
        T::default()
    }
}
pub trait AppTrait<Context = (), Output = ()> {
    type OwnedData;
    fn run(&self, context: &Context, owned_data: Self::OwnedData) -> Output;
    fn meta_app<MetaApp>(self) -> Self
    where
        Self: Sized,
        MetaApp: ConstructFromTrait<Self, Context, Output> + MetaAppTrait<Self, Context, Output>,
    {
        let meta_app = MetaApp::construct_from(&self);
        meta_app.register(self)
    }
    fn register_meta_app<MetaApp>(self, meta_app: MetaApp) -> Self
    where
        Self: Sized,
        MetaApp: MetaAppTrait<Self, Context, Output>,
    {
        meta_app.register(self)
    }
}
