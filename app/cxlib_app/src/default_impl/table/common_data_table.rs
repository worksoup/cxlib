use crate::{ImportExportTrait, NormalTableTrait, StoreError, TableDefinitionTrait};
use cxlib_error_utils::CxlibResultUtils;
use log::warn;
use redb::{Database, ReadTransaction, ReadableTable, WriteTransaction};
use std::{borrow::Borrow, collections::HashMap};

pub struct CommonDataTable;
impl CommonDataTable {
    pub fn iter(r_cxt: &ReadTransaction) -> Result<HashMap<String, String>, StoreError> {
        let r = Self::read(r_cxt)?;
        let mut result = HashMap::new();
        for data in r.iter()? {
            let (k, v) = data?;
            result.insert(k.value(), v.value());
        }
        Ok(result)
    }
    pub fn get(r_cxt: &ReadTransaction, key: &String) -> Result<Option<String>, StoreError> {
        let r = Self::read(r_cxt)?;
        let Some(r) = r.get(key)? else {
            return Ok(None);
        };
        Ok(Some(r.value()))
    }
    pub fn remove(w_cxt: &WriteTransaction, key: &String) -> Result<Option<String>, StoreError> {
        let mut w = Self::write(w_cxt)?;
        let Some(r) = w.remove(key)? else {
            return Ok(None);
        };
        Ok(Some(r.value()))
    }
    pub fn insert(
        w_cxt: &WriteTransaction,
        key: String,
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
    fn import_text<'cxt, Cxt: Borrow<Self::Context<'cxt>>>(db: &Database, _: Cxt, data: &str) {
        let data = toml::from_str::<HashMap<String, String>>(data).log_unwrap();
        let w_cxt = db.begin_write().log_unwrap();
        for (key, value) in data {
            match Self::insert(&w_cxt, key, value) {
                Ok(r) => {
                    if let Some(r) = r {
                        warn!("数据已更新，原数据为：{r:?}");
                    }
                }
                Err(e) => {
                    warn!("common_data 数据行写入出错：`{e}`, 已跳过。");
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
impl NormalTableTrait for CommonDataTable {}
impl TableDefinitionTrait for CommonDataTable {
    type Key = String;
    type Value = String;
    type Context<'cxt> = ();
    const NAME: &'static str = "common_data";
}
