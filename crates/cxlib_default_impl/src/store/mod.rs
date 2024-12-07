mod table;

pub use table::*;

use cxlib_activity::CourseExcludeInfoTrait;
use cxlib_store::{Dir, StorageTableCommandTrait, StorageTrait};
use log::info;
use sqlite::Connection;
use std::fs::File;
use std::ops::Deref;

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
    connection: Connection,
}
impl StorageTrait for DataBase {}
impl Deref for DataBase {
    type Target = Connection;

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
        let connection = Connection::open(db_dir.to_str().unwrap()).unwrap();
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
impl CourseExcludeInfoTrait for DataBase {
    fn is_excluded(&self, id: i64) -> bool {
        ExcludeTable::has_exclude(self, id)
    }

    fn get_excludes(&self) -> Vec<i64> {
        ExcludeTable::get_excludes(self)
    }

    fn exclude(&self, id: i64) {
        ExcludeTable::add_exclude(self, id)
    }

    fn disable_exclude(&self, id: i64) {
        ExcludeTable::delete_exclude(self, id)
    }

    fn update_excludes(&self, excludes: &[i64]) {
        ExcludeTable::update_excludes(self, excludes)
    }
}
