use crate::AppInfo;
use std::path::{Path, PathBuf};

pub trait DirTrait {
    fn get_config_dir(&self) -> PathBuf;
    fn get_database_dir(&self) -> PathBuf;
    fn get_config_file_path<P: AsRef<Path>>(&self, file_name: P) -> PathBuf;
    fn get_json_file_path(&self, account: &str) -> PathBuf;
}

#[derive(Clone)]
pub struct Dir {
    base_dir: PathBuf,
    database_path: PathBuf,
}
impl Default for Dir {
    fn default() -> Self {
        Self::new_with_app_info(&AppInfo::default())
    }
}
impl Dir {
    pub fn new(base_dir: &Path) -> Self {
        let base_dir = base_dir.to_path_buf();
        let database_path = base_dir.join("cx.db");
        Self {
            base_dir,
            database_path,
        }
    }
    pub fn new_with_app_info(app_info: &AppInfo) -> Self {
        let is_testing = std::env::var(app_info.env_arg()).is_ok();
        let binding = directories::ProjectDirs::from(
            app_info.qualifier(),
            app_info.organization(),
            app_info.application(),
        )
        .unwrap();
        let base_dir = if is_testing {
            binding.config_dir().join("test").to_owned()
        } else {
            binding.config_dir().to_owned()
        };
        let _ = std::fs::create_dir_all(base_dir.clone());
        let database_dir = base_dir.join("cx.db");
        Self {
            base_dir,
            database_path: database_dir,
        }
    }
}
impl DirTrait for Dir {
    fn get_config_dir(&self) -> PathBuf {
        self.base_dir.to_path_buf()
    }
    fn get_database_dir(&self) -> PathBuf {
        self.database_path.to_path_buf()
    }
    fn get_config_file_path<P: AsRef<Path>>(&self, file_name: P) -> PathBuf {
        self.base_dir.join(file_name)
    }
    fn get_json_file_path(&self, account: &str) -> PathBuf {
        self.base_dir.join(account.to_string() + ".json")
    }
}
impl From<PathBuf> for Dir {
    fn from(base_dir: PathBuf) -> Self {
        let database_dir = base_dir.join("cx.db");
        Self {
            base_dir,
            database_path: database_dir,
        }
    }
}
