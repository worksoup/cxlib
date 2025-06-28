use crate::{ImportExportTrait, NormalTableTrait, StoreError, TableDefinitionTrait};
use cxlib_error_utils::{CxlibResultUtils, MaybeFatalError};
use log::warn;
use redb::{Database, ReadableTable};
use std::{borrow::Borrow, collections::HashSet};

pub struct ExcludeTable;

impl ExcludeTable {
    pub fn has_exclude(
        table: &impl ReadableTable<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
        id: i64,
    ) -> Result<bool, StoreError> {
        Self::contains_key(table, id)
    }

    pub fn get_excludes(
        table: &impl ReadableTable<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
    ) -> Result<HashSet<i64>, StoreError> {
        let mut excludes = HashSet::new();

        for id in table.iter()? {
            match id {
                Ok(id) => {
                    let id = id.0.value();
                    excludes.insert(id);
                }
                Err(e) => {
                    let e = StoreError::from(e);
                    if e.is_fatal() {
                        return Err(e);
                    }
                }
            }
        }
        Ok(excludes)
    }

    pub fn add_exclude(
        table: &mut redb::Table<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
        id: i64,
    ) -> Result<(), StoreError> {
        table.insert(id, ())?;
        Ok(())
    }

    pub fn delete_exclude(
        table: &mut redb::Table<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
        id: i64,
    ) -> Result<(), StoreError> {
        table.remove(id)?;
        Ok(())
    }
    pub fn remove_all(
        table: &mut redb::Table<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
    ) -> Result<(), StoreError> {
        Ok(table.retain(|_, _| false)?)
    }
    pub fn update_excludes<'a, I: IntoIterator<Item = &'a i64>>(
        table: &mut redb::Table<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
        excludes: I,
    ) -> Result<(), StoreError> {
        Self::remove_all(table)?;
        for exclude in excludes {
            match Self::add_exclude(table, *exclude) {
                Ok(_) => {}
                Err(e) => {
                    if e.is_fatal() {
                        return Err(e);
                    }
                }
            }
        }
        Ok(())
    }
}
impl NormalTableTrait for ExcludeTable {}
impl TableDefinitionTrait for ExcludeTable {
    type Key = i64;
    type Value = ();
    type Context<'cxt> = ();
    const NAME: &'static str = "exclude";
}
impl ImportExportTrait for ExcludeTable {
    fn import_text<'cxt, Cxt: Borrow<Self::Context<'cxt>>>(db: &Database, _: Cxt, content: &str)
    where
        Self: ImportExportTrait,
    {
        struct I64(i64);
        use std::num::ParseIntError;
        use try_from_with_context::TryFromWithContext;
        impl TryFromWithContext<&str> for I64 {
            type Err = ParseIntError;
            type Context<'cxt> = ();
            fn try_from<'cxt, Cxt: Borrow<Self::Context<'cxt>>>(
                s: &str,
                _: Cxt,
            ) -> Result<Self, Self::Err> {
                s.parse::<i64>().map(I64)
            }
        }
        match db.begin_write() {
            Ok(w_cxt) => {
                let data = crate::default_impl::table::parse_lines::<I64, _>(content, ());
                let mut table = Self::write(&w_cxt).log_unwrap();
                for I64(id) in data {
                    match Self::add_exclude(&mut table, id) {
                        Ok(_) => {}
                        Err(e) => {
                            warn!("导入失败：`{e}`.")
                        }
                    }
                }
                drop(table);
                w_cxt.commit().log_unwrap();
            }
            Err(e) => {
                warn!("数据库无法写入：`{e}`。");
            }
        }
    }

    fn export_text(db: &Database) -> String
    where
        Self: ImportExportTrait,
    {
        let Ok(r_cxt) = db.begin_read() else { todo!() };
        let table = Self::read(&r_cxt).log_unwrap();
        let Ok(excludes) = Self::get_excludes(&table) else {
            todo!()
        };
        crate::default_impl::table::to_string_lines(excludes)
    }
}
