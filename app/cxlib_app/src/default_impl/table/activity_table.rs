use crate::{BinCode, ImportExportTrait, NormalTableTrait, StoreError, TableDefinitionTrait};
use bincode::{Decode, Encode};
use cxlib_error_utils::CxlibResultUtils;
use cxlib_internal::types::Activity;
use log::warn;
use redb::{Database, ReadTransaction, ReadableTable, WriteTransaction};
use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, collections::HashMap};
#[derive(
    Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Decode, Encode,
)]
pub struct ActivityTable;
impl ActivityTable {
    pub fn iter(r_cxt: &ReadTransaction) -> Result<HashMap<String, Activity>, StoreError> {
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
        key: impl Borrow<String>,
    ) -> Result<Option<Activity>, StoreError> {
        let r = Self::read(r_cxt)?;
        let Some(r) = r.get(key)? else {
            return Ok(None);
        };
        Ok(Some(r.value()))
    }
    pub fn remove(
        w_cxt: &WriteTransaction,
        key: impl Borrow<String>,
    ) -> Result<Option<Activity>, StoreError> {
        let mut w = Self::write(w_cxt)?;
        let Some(r) = w.remove(key)? else {
            return Ok(None);
        };
        Ok(Some(r.value()))
    }
    pub fn insert(
        w_cxt: &WriteTransaction,
        key: impl Borrow<String>,
        value: impl Borrow<Activity>,
    ) -> Result<Option<Activity>, StoreError> {
        let mut w = Self::write(w_cxt)?;
        let Some(r) = w.insert(key, value)? else {
            return Ok(None);
        };
        Ok(Some(r.value()))
    }
}
impl ImportExportTrait for ActivityTable {
    fn import_text<'cxt, Cxt: Borrow<Self::Context<'cxt>>>(db: &Database, _: Cxt, data: &str) {
        let data = toml::from_str::<HashMap<String, Activity>>(data).log_unwrap();
        let w_cxt = db.begin_write().log_unwrap();
        for (key, value) in data {
            match Self::insert(&w_cxt, key, value) {
                Ok(r) => {
                    if let Some(r) = r {
                        warn!("数据已更新，原数据为：{r:?}");
                    }
                }
                Err(e) => {
                    warn!("activity 数据行写入出错：`{e}`, 已跳过。");
                }
            };
        }
        w_cxt.commit().log_unwrap();
    }

    fn export_text(db: &Database) -> String {
        let export = if let Ok(r_cxt) = db.begin_read() {
            Self::iter(&r_cxt).unwrap_or_default()
        } else {
            HashMap::new()
        };
        toml::to_string_pretty(&export).unwrap_or_default()
    }
}
impl NormalTableTrait for ActivityTable {}
impl TableDefinitionTrait for ActivityTable {
    type Key = String;
    type Value = BinCode<Activity>;
    type Context<'cxt> = ();
    const NAME: &'static str = "activity";
}
