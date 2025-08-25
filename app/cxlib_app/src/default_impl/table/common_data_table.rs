use crate::{
    BinCode, ImportExportTrait, NormalTableTrait, StoreError, TableDefinitionTrait,
    database_guard::DatabaseGuard,
};
use bincode::{Decode, Encode};
use cxlib_error_utils::{CxlibResultUtils, MaybeFatalError};
use log::warn;
use redb::{ReadTransaction, ReadableTable, WriteTransaction};
use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, collections::HashMap};
#[derive(
    Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Decode, Encode,
)]
pub struct KeyType {
    pub block: String,
    pub identifier: String,
    pub key: String,
}
pub struct CommonDataTable;
impl CommonDataTable {
    pub fn iter(r_cxt: &ReadTransaction) -> Result<HashMap<KeyType, String>, StoreError> {
        let r = Self::read(r_cxt)?;
        let mut result = HashMap::new();
        for data in r.iter()? {
            let (k, v) = data?;
            result.insert(k.value(), v.value());
        }
        Ok(result)
    }
    pub fn get(
        r_cxt: &ReadTransaction,
        key: impl Borrow<KeyType>,
    ) -> Result<Option<String>, StoreError> {
        let r = Self::read(r_cxt)?;
        let Some(r) = r.get(key)? else {
            return Ok(None);
        };
        Ok(Some(r.value()))
    }
    pub fn remove(
        w_cxt: &WriteTransaction,
        key: impl Borrow<KeyType>,
    ) -> Result<Option<String>, StoreError> {
        let mut w = Self::write(w_cxt)?;
        let Some(r) = w.remove(key)? else {
            return Ok(None);
        };
        Ok(Some(r.value()))
    }
    pub fn insert(
        w_cxt: &WriteTransaction,
        key: impl Borrow<KeyType>,
        value: String,
    ) -> Result<Option<String>, StoreError> {
        let mut w = Self::write(w_cxt)?;
        let Some(r) = w.insert(key, value)? else {
            return Ok(None);
        };
        Ok(Some(r.value()))
    }
}
impl ImportExportTrait for CommonDataTable {
    fn import_text<'cxt, Cxt: Borrow<Self::Context<'cxt>>>(
        db: &mut DatabaseGuard,
        _: Cxt,
        data: &str,
    ) {
        let data = toml::from_str::<HashMap<KeyType, String>>(data).log_unwrap();
        db.write_once(|w_cxt| {
            for (key, value) in data {
                match Self::insert(w_cxt, key, value) {
                    Ok(r) => {
                        if let Some(r) = r {
                            warn!("数据已更新，原数据为：{r:?}");
                        }
                    }
                    Err(e) => {
                        if e.is_fatal() {
                            warn!("common_data 数据行写入出错：`{e}`, 终止导入。");
                            return Err(e);
                        }
                        warn!("common_data 数据行写入出错：`{e}`, 已跳过。");
                    }
                };
            }
            Ok(())
        })
        .log_unwrap();
    }

    fn export_text(db: &mut DatabaseGuard) -> String {
        let export = db.read_once(Self::iter).log_unwrap_or_default();
        toml::to_string_pretty(&export).unwrap_or_default()
    }
}
impl NormalTableTrait for CommonDataTable {}
impl TableDefinitionTrait for CommonDataTable {
    type Key = BinCode<KeyType>;
    type Value = String;
    type Context<'cxt> = ();
    const NAME: &'static str = "common_data";
}
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct ConfigKey {
    identifier: String,
    key: String,
}
pub trait ConfigTrait: Sized + Eq + std::hash::Hash {
    const BLOCK: &'static str;
    fn into_identifier_key(self) -> (String, String);
    fn from_identifier_key(identifier_key: (String, String)) -> Self;
    #[inline]
    fn get(self, r_cxt: &ReadTransaction) -> Result<Option<String>, StoreError> {
        let (identifier, key) = self.into_identifier_key();
        let key = KeyType {
            block: Self::BLOCK.to_owned(),
            identifier,
            key,
        };
        CommonDataTable::get(r_cxt, key)
    }
    #[inline]
    fn remove(self, w_cxt: &WriteTransaction) -> Result<Option<String>, StoreError> {
        let (identifier, key) = self.into_identifier_key();
        let key = KeyType {
            block: Self::BLOCK.to_owned(),
            identifier,
            key,
        };
        CommonDataTable::remove(w_cxt, key)
    }
    #[inline]
    fn insert(self, w_cxt: &WriteTransaction, value: String) -> Result<Option<String>, StoreError> {
        let (identifier, key) = self.into_identifier_key();
        let key = KeyType {
            block: Self::BLOCK.to_owned(),
            identifier,
            key,
        };
        CommonDataTable::insert(w_cxt, key, value)
    }
    #[inline]
    fn iter(r_cxt: &ReadTransaction) -> Result<HashMap<Self, String>, StoreError> {
        let r = CommonDataTable::read(r_cxt)?;
        let mut result = HashMap::new();
        for data in r.iter()? {
            let (k, v) = data?;
            let k = k.value();
            result.insert(Self::from_identifier_key((k.identifier, k.key)), v.value());
        }
        Ok(result)
    }
}
impl ConfigTrait for ConfigKey {
    const BLOCK: &'static str = "config";

    #[inline]
    fn into_identifier_key(self) -> (String, String) {
        let Self { identifier, key } = self;
        (identifier, key)
    }

    #[inline]
    fn from_identifier_key((identifier, key): (String, String)) -> Self {
        Self { identifier, key }
    }
}
