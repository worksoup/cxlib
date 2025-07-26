use crate::{
    BinCode, CourseTable, ImportExportTrait, NormalTableTrait, StoreError, TableDefinitionTrait,
};
use cxlib_error_utils::CxlibResultUtils;
use cxlib_internal::types::Activity;
use log::warn;
use redb::{Database, ReadableTable, Table, WriteTransaction};
use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
};
pub struct ActivityTable;
impl ActivityTable {
    pub fn iter(
        table: &impl ReadableTable<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
    ) -> Result<HashMap<String, (Activity, Vec<String>)>, StoreError> {
        let mut result = HashMap::new();
        for data in table.iter()? {
            let (k, v) = data?;
            result.insert(k.value(), v.value());
        }
        Ok(result)
    }
    pub fn get(
        table: &impl ReadableTable<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
        key: impl Borrow<String>,
    ) -> Result<Option<(Activity, Vec<String>)>, StoreError> {
        let Some(r) = table.get(key)? else {
            return Ok(None);
        };
        Ok(Some(r.value()))
    }
    pub fn remove(
        table: &mut Table<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
        key: impl Borrow<String>,
    ) -> Result<Option<(Activity, Vec<String>)>, StoreError> {
        let Some(r) = table.remove(key)? else {
            return Ok(None);
        };
        Ok(Some(r.value()))
    }
    pub fn insert(
        table: &mut Table<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
        key: impl Borrow<String>,
        value: impl Borrow<(Activity, Vec<String>)>,
    ) -> Result<Option<(Activity, Vec<String>)>, StoreError> {
        let Some(r) = table.insert(key, value)? else {
            return Ok(None);
        };
        Ok(Some(r.value()))
    }
    pub fn update_status(
        activity_table: &mut Table<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
        active_id: impl Borrow<String>,
        status: i32,
    ) -> Result<bool, StoreError> {
        if let Some((old_activity, old_data)) = Self::get(activity_table, active_id.borrow())? {
            let mut old_activity = old_activity;
            old_activity.set_status_code(status);
            let r = (old_activity, old_data);
            activity_table.insert(active_id, &r)?;
            return Ok(true);
        };
        Ok(false)
    }
    pub fn merge(
        w_cxt: &WriteTransaction,
        key: impl Borrow<String>,
        (activity, value): (Activity, Vec<String>),
    ) -> Result<(Activity, Vec<String>), StoreError> {
        let mut activity_table = ActivityTable::write(w_cxt)?;
        let users = if let Some((_, old_data)) = Self::get(&activity_table, key.borrow())? {
            let mut old_data = old_data.into_iter().collect::<HashSet<_>>();
            for v in value {
                old_data.insert(v.to_owned());
            }
            old_data.into_iter().collect::<Vec<_>>()
        } else {
            let course_table = CourseTable::write(w_cxt);
            if let Ok(mut course_table) = course_table
                && let Ok(a) = {
                    let course = activity.course().course();
                    CourseTable::update_recently_used_time(
                        &mut course_table,
                        course,
                        activity.start_time_mills(),
                    )
                }
                && a
            {
            } else {
                warn!("无法更新课程最近活动时间。");
            }
            value
        };
        let r = (activity, users);
        activity_table.insert(key, &r)?;
        drop(activity_table);
        Ok(r)
    }
}
impl ImportExportTrait for ActivityTable {
    fn import_text<'cxt, Cxt: Borrow<Self::Context<'cxt>>>(db: &Database, _: Cxt, data: &str) {
        let data = toml::from_str::<HashMap<String, (Activity, Vec<String>)>>(data).log_unwrap();
        let w_cxt = db.begin_write().log_unwrap();
        let mut table = Self::write(&w_cxt).log_unwrap();
        for (key, value) in data {
            match Self::insert(&mut table, key, value) {
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
        drop(table);
        w_cxt.commit().log_unwrap();
    }

    fn export_text(db: &Database) -> String {
        let export = if let Ok(r_cxt) = db.begin_read()
            && let Ok(table) = Self::read(&r_cxt)
        {
            Self::iter(&table).unwrap_or_default()
        } else {
            HashMap::new()
        };
        toml::to_string_pretty(&export).unwrap_or_default()
    }
}
impl NormalTableTrait for ActivityTable {}
impl TableDefinitionTrait for ActivityTable {
    type Key = BinCode<String>;
    type Value = BinCode<(Activity, Vec<String>)>;
    type Context<'cxt> = ();
    const NAME: &'static str = "activity";
}
