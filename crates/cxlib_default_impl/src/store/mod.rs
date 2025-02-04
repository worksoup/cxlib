mod table;

pub use cxlib_error::StoreError;
pub use table::*;

use cxlib_store::{Dir, StorageTableCommandTrait, StorageTrait};
use log::info;
use sqlite::{Connection, ConnectionThreadSafe};
use std::{fs::File, ops::Deref};

pub trait DataBaseTableTrait: StorageTableCommandTrait<DataBase> {
    const TABLE_ARGS: &'static str;
    const TABLE_NAME: &'static str;
    fn init(db: &DataBase) {
        if !Self::is_existed(db) {
            db.execute(format!(
                "CREATE TABLE {} ({});",
                Self::TABLE_NAME,
                Self::TABLE_ARGS
            ))
            .unwrap();
        }
    }
    fn is_existed(db: &DataBase) -> bool {
        let mut query = db
            .prepare(format!(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='{}';",
                Self::TABLE_NAME
            ))
            .unwrap();
        query.next().unwrap();
        query.read::<i64, _>(0).unwrap() == 1
    }
    fn delete(db: &DataBase) {
        let mut query = db
            .prepare(format!("DELETE FROM {};", Self::TABLE_NAME))
            .unwrap();
        query.next().unwrap();
        info!("已删除数据表 {}。", Self::TABLE_NAME);
    }
    fn import(db: &DataBase, content: &str) {
        <Self as StorageTableCommandTrait<DataBase>>::import(db, content);
    }
    fn export(db: &DataBase) -> String {
        <Self as StorageTableCommandTrait<DataBase>>::export(db)
    }
}
pub struct DataBase {
    connection: ConnectionThreadSafe,
}
impl StorageTrait for DataBase {}
impl Deref for DataBase {
    type Target = ConnectionThreadSafe;

    fn deref(&self) -> &Self::Target {
        &self.connection
    }
}
// self
impl DataBase {
    pub fn new() -> Self {
        let db_dir = Dir::get_database_dir();
        if db_dir.metadata().is_err() {
            File::create(db_dir.clone()).unwrap();
        }
        let connection = Connection::open_thread_safe(db_dir.to_str().unwrap()).unwrap();
        Self { connection }
    }
    pub fn add_table<T: DataBaseTableTrait>(&self) {
        <T as DataBaseTableTrait>::init(self)
    }
}
impl Default for DataBase {
    fn default() -> Self {
        Self::new()
    }
}
