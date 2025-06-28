pub use cxlib_app::*;
pub use cxlib_internal::*;

pub fn run() {
    use cxlib_app::{
        AccountCmdApp, AccountsCmdApp, AppTrait, CmdApp, CmdAppContext, CoursesCmdApp,
        LocationCmdApp, LocationsCmdApp, SignMainApp, WhereIsConfigCmdApp,
    };
    fn init(self_: &CmdApp<CmdAppContext>) -> (CmdAppContext, ()) {
        if let Some(captcha_type) = std::env::var("CX_CAPTCHA_TYPE")
            .ok()
            .and_then(|s| s.parse().ok())
        {
            let _ = CaptchaType::set_global_default(&captcha_type);
        }
        let env = env_logger::Env::default().filter_or("RUST_LOG", "info");
        let mut builder = env_logger::Builder::from_env(env);
        builder.target(env_logger::Target::Stderr);
        builder.init();
        Dir::set_config_dir_info(
            "TEST_CXSIGN",
            "up.workso",
            "Worksoup",
            env!("CARGO_PKG_NAME"),
        );
        let db = Database::default();
        (CmdAppContext::new(db, self_.command().clone(), 1, 2), ())
    }
    let cmd_app = CmdApp::new(clap::command!())
        .main_app::<SignMainApp>(Default::default())
        .meta_app(AccountCmdApp)
        .meta_app(AccountsCmdApp)
        .meta_app(CoursesCmdApp::default())
        .meta_app(LocationCmdApp::default())
        .meta_app(LocationsCmdApp::default())
        .meta_app(WhereIsConfigCmdApp::default());
    #[cfg(feature = "completion")]
    let cmd_app = cmd_app.meta_app(cxlib::CompletionCmdApp::default());
    cmd_app.init_and_run(init)
}
