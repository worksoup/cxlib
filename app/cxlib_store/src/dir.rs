use crate::AppInfo;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ConfigDir(PathBuf);
impl Default for ConfigDir {
    #[inline]
    fn default() -> Self {
        Self::new(&AppInfo::default())
    }
}
impl ConfigDir {
    pub fn new(app_info: &AppInfo) -> Self {
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
        Self(base_dir)
    }
    #[inline]
    pub fn get_config_dir(&self) -> &Path {
        &self.0
    }
}
impl From<PathBuf> for ConfigDir {
    #[inline]
    fn from(config_dir: PathBuf) -> Self {
        Self(config_dir)
    }
}
