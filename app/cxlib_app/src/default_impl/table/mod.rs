//! redb 使用细节。

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
    Database, Key, MultimapTable, MultimapTableDefinition, ReadOnlyMultimapTable, ReadOnlyTable,
    ReadTransaction, ReadableMultimapTable, ReadableTable, Table, TableDefinition,
    UntypedMultimapTableHandle, UntypedTableHandle, Value, WriteTransaction,
};
use std::borrow::Borrow;

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
        w_cxt: &WriteTransaction,
    ) -> Result<MultimapTable<Self::Key, Self::Value>, StoreError> {
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
    fn write(w_cxt: &WriteTransaction) -> Result<Table<Self::Key, Self::Value>, StoreError> {
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
    fn import_text<'cxt, Cxt: Borrow<Self::Context<'cxt>>>(db: &Database, cxt: Cxt, content: &str);
    fn export_text(db: &Database) -> String;
}
pub mod database_guard {
    //! 也许能防止一个块内试图同时获取读事务与写事务。
    use std::ops::Deref;

    use redb::{Database, ReadTransaction, WriteTransaction};
    pub struct DatabaseGuard<'a>(&'a Database);
    impl<'b> DatabaseGuard<'b> {
        pub fn new(db: &'b Database) -> Self {
            Self(db)
        }
        pub fn read<'a, T, E, F: FnOnce(&ReadTransaction) -> Result<T, E>>(
            &'a self,
            f: F,
        ) -> Result<ReadAccessGuard<'a, 'b, T>, E>
        where
            'b: 'a,
            E: From<redb::TransactionError>,
        {
            let r_cxt = self.begin_read()?;
            let t = f(&r_cxt)?;
            Ok(ReadAccessGuard(self, r_cxt, t))
        }
        pub fn write_map_err<
            'a,
            T,
            N,
            E,
            F: FnOnce(&WriteTransaction) -> Result<T, N>,
            M: FnOnce(E) -> N,
        >(
            &'a mut self,
            f: F,
            m: M,
        ) -> Result<WriteAccessGuard<'a, 'b, T>, N>
        where
            'b: 'a,
            E: From<redb::TransactionError> + From<redb::CommitError>,
        {
            match self.begin_write() {
                Ok(w_cxt) => {
                    let t = f(&w_cxt)?;
                    w_cxt.commit().map_err(E::from).map_err(m)?;
                    Ok(WriteAccessGuard(self, t))
                }
                Err(e) => Err(m(E::from(e))),
            }
        }
        pub fn write<'a, T, E, F: FnOnce(&WriteTransaction) -> Result<T, E>>(
            &'a mut self,
            f: F,
        ) -> Result<WriteAccessGuard<'a, 'b, T>, E>
        where
            'b: 'a,
            E: From<redb::TransactionError> + From<redb::CommitError>,
        {
            let w_cxt = self.begin_write()?;
            let t = f(&w_cxt)?;
            w_cxt.commit()?;
            Ok(WriteAccessGuard(self, t))
        }
    }
    impl Deref for DatabaseGuard<'_> {
        type Target = Database;

        fn deref(&self) -> &Self::Target {
            self.0
        }
    }
    pub struct ReadAccessGuard<'a, 'b: 'a, T>(&'a DatabaseGuard<'b>, ReadTransaction, T);
    impl<T> ReadAccessGuard<'_, '_, T> {
        pub fn into_inner(self) -> T {
            self.2
        }
    }
    impl<'a, 'b: 'a, R> ReadAccessGuard<'a, 'b, R> {
        pub fn read<T, E, F: FnOnce(&ReadTransaction) -> Result<T, E>>(
            self,
            f: F,
        ) -> Result<(R, ReadAccessGuard<'a, 'b, T>), E>
        where
            'b: 'a,
            E: From<redb::TransactionError>,
        {
            let Self(g, r_cxt, r) = self;
            let t = f(&r_cxt)?;
            Ok((r, ReadAccessGuard(g, r_cxt, t)))
        }
    }
    impl<T> Deref for ReadAccessGuard<'_, '_, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self.2
        }
    }
    impl<T> AsRef<ReadTransaction> for ReadAccessGuard<'_, '_, T> {
        fn as_ref(&self) -> &ReadTransaction {
            &self.1
        }
    }
    pub struct WriteAccessGuard<'a, 'b: 'a, T>(#[allow(dead_code)] &'a mut DatabaseGuard<'b>, T);
    impl<T> Deref for WriteAccessGuard<'_, '_, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self.1
        }
    }
    impl<T> WriteAccessGuard<'_, '_, T> {
        pub fn into_inner(self) -> T {
            self.1
        }
    }
}
