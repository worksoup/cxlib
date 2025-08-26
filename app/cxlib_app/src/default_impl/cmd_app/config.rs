use clap::{ArgMatches, FromArgMatches, Parser};
use cxlib_error_utils::CxlibResultUtils;

use crate::{
    AppTrait, CmdMetaAppTrait, CommonDataTable, ConfigKey, ConfigTrait, NormalTableTrait,
    StoreError, database_guard::DatabaseGuard,
};

#[derive(Parser, Debug, Clone)]
// TODO: build.rs 中通过环境变量设置 alias.
#[command(name = "config", alias = "conf")]
/// 列出所有账号。
pub enum ConfigParser {
    /// 获取 `key` 对应的配置项值。
    Get {
        /// `key`, 格式为 `identifier.key`, `identifier` 默认为 "default".
        #[arg(short, long)]
        key: String,
    },
    /// 设置 `key` 对应的配置项值。
    Set {
        /// `key`, 格式为 `identifier.key`, `identifier` 默认为 "default".
        #[arg(short, long)]
        key: String,
        #[arg(short, long)]
        value: String,
    },
    /// 取消设置 `key` 对应的配置项值。
    Unset {
        /// `key`, 格式为 `identifier.key`, `identifier` 默认为 "default".
        #[arg(short, long)]
        key: String,
    },
}
impl ConfigParser {
    pub fn parse_key(key: String) -> ConfigKey {
        fn is_valid(s: &str) -> bool {
            for c in s.chars() {
                if c == '.' {
                    return false;
                }
            }
            true
        }
        let identifier_key = key.split_once('.');
        if let Some((identifier, key)) = identifier_key {
            assert!(is_valid(identifier));
            assert!(is_valid(key));
            ConfigKey::from_identifier_key((identifier.to_owned(), key.to_owned()))
        } else {
            assert!(is_valid(&key));
            ConfigKey::from_identifier_key(("general".to_owned(), key.to_owned()))
        }
    }
}
pub struct ConfigCmdApp {}
impl Default for ConfigCmdApp {
    #[inline]
    fn default() -> Self {
        Self {}
    }
}
impl<Context> AppTrait<Context> for ConfigCmdApp
where
    Context: AsRef<DatabaseGuard>,
{
    type OwnedData = ConfigParser;

    #[inline]
    fn run(&self, cxt: &Context, owned_data: ConfigParser) {
        let db_g = cxt.as_ref();
        let mut db_g = db_g.clone();

        match owned_data {
            ConfigParser::Get { key } => {
                match db_g.read_once(CommonDataTable::read) {
                    Ok(_) => {}
                    Err(StoreError::TableError(redb::TableError::TableDoesNotExist(_))) => {
                        db_g.write_once(|w_cxt| {
                            let _ = CommonDataTable::write(w_cxt)?;
                            Ok::<_, StoreError>(())
                        })
                        .log_unwrap();
                    }
                    e => {
                        let _ = e.log_unwrap();
                    }
                };
                let result = db_g
                    .read_once(|r_cxt| {
                        let key = ConfigParser::parse_key(key.clone());
                        key.get(r_cxt)
                    })
                    .log_unwrap();
                if let Some(result) = result {
                    log::info!("键：`{key}` 对应的值为:\n");
                    log::info!("\t`{result}`.")
                } else {
                    log::warn!("键：`{key}` 对应的值不存在。")
                }
            }
            ConfigParser::Set { key, value } => {
                db_g.write_once(|w_cxt| {
                    let key = ConfigParser::parse_key(key);
                    key.insert(w_cxt, value)
                })
                .log_unwrap();
            }
            ConfigParser::Unset { key } => {
                db_g.write_once(|w_cxt| {
                    let key = ConfigParser::parse_key(key);
                    key.remove(w_cxt)
                })
                .log_unwrap();
            }
        }
    }
}
impl<Context, OwnedData> CmdMetaAppTrait<Context, OwnedData> for ConfigCmdApp
where
    Context: AsRef<DatabaseGuard>,
{
    #[inline]
    fn read_owned_data(
        &self,
        _: &Context,
        matches: &[&ArgMatches],
    ) -> <Self as AppTrait<Context, ()>>::OwnedData {
        ConfigParser::from_arg_matches(matches.last().unwrap()).unwrap()
    }
}
