mod cmd_app;
mod default_impl;

pub use cmd_app::*;
pub use default_impl::*;

use cxlib_internal as cxlib;
pub trait MetaAppTrait<App: AppTrait<Context, Output>, Context = (), Output = ()>:
    AppTrait<Context, Output>
{
    fn register(self, app: App) -> App
    where
        Self: Sized;
}
pub trait AppTrait<Context = (), Output = ()> {
    type OwnedData;
    fn run(&self, context: &Context, owned_data: Self::OwnedData) -> Output;
    fn meta_app<MetaApp: MetaAppTrait<Self, Context, Output>>(self, meta_app: MetaApp) -> Self
    where
        Self: Sized,
    {
        meta_app.register(self)
    }
}
