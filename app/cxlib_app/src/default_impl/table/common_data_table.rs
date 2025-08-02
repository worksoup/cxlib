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
