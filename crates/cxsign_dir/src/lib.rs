#![feature(sync_unsafe_cell)]

use std::path::{Path, PathBuf};
const UNINITIALIZED: usize = 0;
const INITIALIZING: usize = 1;
const INITIALIZED: usize = 2;
mod config_dir_info_state {
    use crate::{INITIALIZED, INITIALIZING, UNINITIALIZED};
    use std::cell::SyncUnsafeCell;
    use std::sync::atomic::{AtomicUsize, Ordering};

    pub static CONFIG_DIR_INFO: SyncUnsafeCell<(&str, &str, &str, &str)> =
        SyncUnsafeCell::new(("TEST_CXSIGN", "up.workso", "Worksoup", "cxsign"));
    static STATE: AtomicUsize = AtomicUsize::new(0);
    pub fn set_config_dir_info(
        env_arg: &'static str,
        qualifier: &'static str,
        organization: &'static str,
        application: &'static str,
    ) {
        let old_state = match STATE.compare_exchange(
            UNINITIALIZED,
            INITIALIZING,
            Ordering::SeqCst,
            Ordering::SeqCst,
        ) {
            Ok(s) | Err(s) => s,
        };
        match old_state {
            UNINITIALIZED => {
                unsafe { *CONFIG_DIR_INFO.get() = (env_arg, qualifier, organization, application) }
                STATE.store(INITIALIZED, Ordering::SeqCst);
            }
            INITIALIZING => {
                while STATE.load(Ordering::SeqCst) == crate::INITIALIZING {
                    std::hint::spin_loop()
                }
            }
            _ => (),
        }
    }
}
mod config_dir_state {
    use crate::{Dir, INITIALIZED, INITIALIZING, UNINITIALIZED};
    use std::cell::SyncUnsafeCell;
    use std::sync::atomic::{AtomicUsize, Ordering};

    pub static CONFIG_DIR: SyncUnsafeCell<Option<&Dir>> = SyncUnsafeCell::new(None);
    static STATE: AtomicUsize = AtomicUsize::new(0);

    pub fn uninit() -> bool {
        let state = STATE.load(Ordering::SeqCst);
        match state {
            UNINITIALIZED => true,
            INITIALIZING => {
                while STATE.load(Ordering::SeqCst) == INITIALIZING {
                    println!("adasd");
                    std::hint::spin_loop()
                }
                false
            }
            _ => false,
        }
    }
    pub fn set_config_dir(dir: Box<Dir>) {
        let old_state = match STATE.compare_exchange(
            UNINITIALIZED,
            INITIALIZING,
            Ordering::SeqCst,
            Ordering::SeqCst,
        ) {
            Ok(s) | Err(s) => s,
        };
        match old_state {
            UNINITIALIZED => {
                unsafe { *CONFIG_DIR.get() = Some(Box::leak(dir)) }
                STATE.store(INITIALIZED, Ordering::SeqCst);
            }
            INITIALIZING => {
                while STATE.load(Ordering::SeqCst) == crate::INITIALIZING {
                    std::hint::spin_loop()
                }
            }
            _ => (),
        }
    }
}

#[derive(Clone)]
pub struct Dir {
    base_dir: PathBuf,
    database_dir: PathBuf,
}
impl Dir {
    pub fn set_config_dir_info(
        env_arg: &'static str,
        qualifier: &'static str,
        organization: &'static str,
        application: &'static str,
    ) {
        config_dir_info_state::set_config_dir_info(env_arg, qualifier, organization, application);
    }
    fn set_default_config_dir() {
        let (env_arg, qualifier, organization, application) =
            unsafe { *config_dir_info_state::CONFIG_DIR_INFO.get() };
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
        config_dir_state::set_config_dir(dir);
    }
    pub fn new(base_dir: &Path) -> Self {
        let base_dir = base_dir.to_path_buf();
        let database_dir = base_dir.join("cx.db");
        Self {
            base_dir,
            database_dir,
        }
    }
    unsafe fn get_dir_unchecked() -> &'static Dir {
        unsafe { (*config_dir_state::CONFIG_DIR.get()).unwrap_unchecked() }
    }
    pub fn get_config_dir() -> PathBuf {
        if config_dir_state::uninit() {
            Self::set_default_config_dir()
        }
        unsafe { Self::get_dir_unchecked() }.base_dir.to_path_buf()
    }
    pub fn get_database_dir() -> PathBuf {
        if config_dir_state::uninit() {
            Self::set_default_config_dir()
        }
        unsafe { Self::get_dir_unchecked() }
            .database_dir
            .to_path_buf()
    }
    pub fn get_json_file_path(account: &str) -> PathBuf {
        if config_dir_state::uninit() {
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
