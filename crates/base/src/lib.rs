use std::path::PathBuf;

pub mod activity;
pub mod course;
pub mod location;
pub mod photo;
pub mod protocol;
pub mod user;
pub mod utils;

lazy_static::lazy_static! {
    static ref CONFIG_DIR: PathBuf = {
        let is_testing = std::env::var("TEST_NEWSIGN").is_ok();
        let binding = directories::ProjectDirs::from("rt.lea", "worksoup", "newsign").unwrap();
        let dir = if is_testing {
            binding.config_dir().join("test").to_owned()
        } else {
            binding.config_dir().to_owned()
        };
        let _ = std::fs::create_dir_all(dir.clone());
        dir
    };
    static ref DATABASE_DIR: PathBuf = CONFIG_DIR.join("cx.db");
}
pub fn get_config_dir() -> PathBuf {
    CONFIG_DIR.to_path_buf()
}
pub fn get_database_dir() -> PathBuf {
    DATABASE_DIR.to_path_buf()
}
pub fn get_json_file_path(account: &str) -> PathBuf {
    CONFIG_DIR.join(account.to_string() + ".json")
}
