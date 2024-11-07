#![feature(sync_unsafe_cell)]

use onceinit::{OnceInit, OnceInitState, StaticDefault};
use std::ops::Deref;
use std::path::{Path, PathBuf};

struct ConfigDirInfo {
    env_arg: &'static str,
    qualifier: &'static str,
    organization: &'static str,
    application: &'static str,
}
static DEFAULT_CONFIG_DIR_INFO: ConfigDirInfo = ConfigDirInfo {
    env_arg: "TEST_CXSIGN",
    qualifier: "up.workso",
    organization: "Worksoup",
    application: "cxsign",
};
impl StaticDefault for ConfigDirInfo {
    fn static_default() -> &'static Self {
        &DEFAULT_CONFIG_DIR_INFO
    }
}
static CONFIG_DIR_INFO: OnceInit<ConfigDirInfo> = OnceInit::new();

static CONFIG_DIR: OnceInit<Dir> = OnceInit::new();

fn uninit() -> bool {
    match CONFIG_DIR.get_state() {
        OnceInitState::INITIALIZED => false,
        OnceInitState::INITIALIZING => {
            while let OnceInitState::INITIALIZING = CONFIG_DIR.get_state() {
                std::hint::spin_loop()
            }
            false
        }
        _ => true,
    }
}

#[derive(Clone)]
pub struct Dir {
    base_dir: PathBuf,
    database_dir: PathBuf,
}
impl Dir {
    pub fn new(base_dir: &Path) -> Self {
        let base_dir = base_dir.to_path_buf();
        let database_dir = base_dir.join("cx.db");
        Self {
            base_dir,
            database_dir,
        }
    }
    pub fn set_config_dir_info(
        env_arg: &'static str,
        qualifier: &'static str,
        organization: &'static str,
        application: &'static str,
    ) {
        let data = Box::new(ConfigDirInfo {
            env_arg,
            qualifier,
            organization,
            application,
        });
        let _ = CONFIG_DIR_INFO.set_boxed_data(data);
    }
    fn set_default_config_dir() {
        let ConfigDirInfo {
            env_arg,
            qualifier,
            organization,
            application,
        } = { CONFIG_DIR_INFO.deref().to_owned() };
        let is_testing = std::env::var(env_arg).is_ok();
        let binding = directories::ProjectDirs::from(qualifier, organization, application).unwrap();
        let base_dir = if is_testing {
            binding.config_dir().join("test").to_owned()
        } else {
            binding.config_dir().to_owned()
        };
        let _ = std::fs::create_dir_all(base_dir.clone());
        let database_dir = base_dir.join("cx.db");
        let dir = Box::new(Self {
            base_dir,
            database_dir,
        });
        Self::set_config_dir(dir);
    }
    pub fn set_config_dir(dir: Box<Self>) {
        let _ = CONFIG_DIR.set_boxed_data(dir);
    }
    unsafe fn get_dir_unchecked() -> &'static Dir {
        CONFIG_DIR.get_data_unchecked()
    }
    pub fn get_config_dir() -> PathBuf {
        if uninit() {
            Self::set_default_config_dir()
        }
        unsafe { Self::get_dir_unchecked() }.base_dir.to_path_buf()
    }
    pub fn get_database_dir() -> PathBuf {
        if uninit() {
            Self::set_default_config_dir()
        }
        unsafe { Self::get_dir_unchecked() }
            .database_dir
            .to_path_buf()
    }
    pub fn get_json_file_path(account: &str) -> PathBuf {
        if uninit() {
            Self::set_default_config_dir()
        }
        unsafe { Self::get_dir_unchecked() }
            .base_dir
            .join(account.to_string() + ".json")
    }
}
impl From<PathBuf> for Dir {
    fn from(base_dir: PathBuf) -> Self {
        let database_dir = base_dir.join("cx.db");
        Self {
            base_dir,
            database_dir,
        }
    }
}
