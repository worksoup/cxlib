//! redb 使用细节。
//!
//! RTrans 会获取最近提交的内容（但也许没有更新机制？）
//!
//! WTrans 会在有其他 WTrans 时阻塞线程。

mod account_table;
mod activity_table;
mod alias_table;
mod common_data_table;
mod course_table;
mod error;
mod exclude_table;
mod location_table;
// mod table_base;
mod utils;

pub mod database_guard;

pub use account_table::*;
pub use activity_table::*;
pub use alias_table::*;
pub use common_data_table::*;
pub use course_table::*;
pub use error::*;
pub use exclude_table::*;
pub use location_table::*;
// pub use table_base::*;
pub use utils::*;

use redb::{
    Key, MultimapTable, MultimapTableDefinition, ReadOnlyMultimapTable, ReadOnlyTable,
    ReadTransaction, ReadableMultimapTable, ReadableTable, Table, TableDefinition,
    UntypedMultimapTableHandle, UntypedTableHandle, Value, WriteTransaction,
};
use std::borrow::Borrow;

use crate::database_guard::DatabaseGuard;

pub trait TableDefinitionTrait {
    type Key: Key + 'static;
    type Value: Value + 'static;
    type Context<'cxt>: ?Sized;
    const NAME: &'static str;
}
pub trait MultimapTableTrait: TableDefinitionTrait
where
    <Self as TableDefinitionTrait>::Value: Key,
{
    const DEFINITION: MultimapTableDefinition<'static, Self::Key, Self::Value> =
        MultimapTableDefinition::new(Self::NAME);
    fn contains_key<'a>(
        table: &impl ReadableMultimapTable<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
        key: impl Borrow<<Self::Key as Value>::SelfType<'a>>,
    ) -> Result<bool, StoreError> {
        Ok(table.get(key).map(|k| !k.is_empty())?)
    }
    fn remove_value<'a>(
        table: &mut redb::MultimapTable<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
        key: impl Borrow<<Self::Key as Value>::SelfType<'a>>,
        value: impl Borrow<<Self::Value as Value>::SelfType<'a>>,
    ) -> Result<bool, StoreError> {
        Ok(table.remove(key, value)?)
    }
    fn write(
        w_cxt: &'_ WriteTransaction,
    ) -> Result<MultimapTable<'_, Self::Key, Self::Value>, StoreError> {
        Ok(w_cxt.open_multimap_table(Self::DEFINITION)?)
    }
    fn list(
        r_cxt: &ReadTransaction,
    ) -> Result<impl Iterator<Item = UntypedMultimapTableHandle>, StoreError> {
        Ok(r_cxt.list_multimap_tables()?)
    }
    fn read(
        r_cxt: &ReadTransaction,
    ) -> Result<ReadOnlyMultimapTable<Self::Key, Self::Value>, StoreError> {
        Ok(r_cxt.open_multimap_table(Self::DEFINITION)?)
    }
    fn delete(w_cxt: &WriteTransaction) -> Result<bool, StoreError> {
        Ok(w_cxt.delete_multimap_table(Self::DEFINITION)?)
    }
}
pub trait NormalTableTrait: TableDefinitionTrait
where
    <Self as TableDefinitionTrait>::Value: Key,
{
    const DEFINITION: TableDefinition<'static, Self::Key, Self::Value> =
        TableDefinition::new(Self::NAME);
    fn contains_key<'a>(
        table: &impl ReadableTable<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
        key: impl Borrow<<Self::Key as Value>::SelfType<'a>>,
    ) -> Result<bool, StoreError> {
        Ok(table.get(key).map(|k| k.is_some())?)
    }
    fn write(w_cxt: &'_ WriteTransaction) -> Result<Table<'_, Self::Key, Self::Value>, StoreError> {
        Ok(w_cxt.open_table(Self::DEFINITION)?)
    }
    fn list(
        r_cxt: &ReadTransaction,
    ) -> Result<impl Iterator<Item = UntypedTableHandle>, StoreError> {
        Ok(r_cxt.list_tables()?)
    }
    fn read(r_cxt: &ReadTransaction) -> Result<ReadOnlyTable<Self::Key, Self::Value>, StoreError> {
        Ok(r_cxt.open_table(Self::DEFINITION)?)
    }
    fn delete(w_cxt: &WriteTransaction) -> Result<bool, StoreError> {
        Ok(w_cxt.delete_table(Self::DEFINITION)?)
    }
}
pub trait ImportExportTrait: TableDefinitionTrait {
    fn import_text<'cxt, Cxt: Borrow<Self::Context<'cxt>>>(
        db: &mut DatabaseGuard,
        cxt: Cxt,
        content: &str,
    );
    fn export_text(db: &mut DatabaseGuard) -> String;
}
