use crate::store::{DataBase, DataBaseTableTrait};
use cxlib_store::StorageTableCommandTrait;
use log::warn;
use std::collections::HashSet;

pub struct ExcludeTable;

impl ExcludeTable {
    pub fn has_exclude(db: &DataBase, id: i64) -> bool {
        let mut query = db
            .prepare(format!(
                "SELECT count(*) FROM {} WHERE id=?;",
                Self::TABLE_NAME
            ))
            .unwrap();
        query.bind((1, id)).unwrap();
        query.next().unwrap();
        query.read::<i64, _>(0).unwrap() > 0
    }

    pub fn get_excludes(db: &DataBase) -> HashSet<i64> {
        let mut query = db
            .prepare(format!("SELECT * FROM {};", Self::TABLE_NAME))
            .unwrap();
        let mut excludes = HashSet::new();
        for c in query.iter() {
            if let Ok(row) = c {
                let id = row.read("id");
                excludes.insert(id);
            } else {
                warn!("账号解析行出错：{c:?}.");
            }
        }
        excludes
    }

    pub fn add_exclude(db: &DataBase, id: i64) {
        let mut query = db
            .prepare(format!("INSERT INTO {}(id) values(:id);", Self::TABLE_NAME))
            .unwrap();
        query
            .bind::<&[(_, sqlite::Value)]>(&[(":id", id.into())][..])
            .unwrap();
        let _ = query.next();
    }

    pub fn delete_exclude(db: &DataBase, id: i64) {
        if Self::has_exclude(db, id) {
            let mut query = db
                .prepare(format!("DELETE FROM {} WHERE id=?;", Self::TABLE_NAME))
                .unwrap();
            query.bind((1, id)).unwrap();
            query.next().unwrap();
        }
    }

    pub fn update_excludes<'a, I: IntoIterator<Item = &'a i64>>(db: &DataBase, excludes: I) {
        Self::delete(db);
        for exclude in excludes {
            Self::add_exclude(db, *exclude);
        }
    }
}
impl StorageTableCommandTrait<DataBase> for ExcludeTable {
    fn init(storage: &DataBase) {
        <Self as DataBaseTableTrait>::init(storage);
    }
    fn uninit(storage: &DataBase) -> bool {
        !Self::is_existed(storage)
    }
    fn clear(storage: &DataBase) {
        Self::delete(storage);
    }
    fn import(storage: &DataBase, content: &str) {
        <Self as DataBaseTableTrait>::import(storage, content);
    }
    fn export(storage: &DataBase) -> String {
        <Self as DataBaseTableTrait>::export(storage)
    }
}
impl DataBaseTableTrait for ExcludeTable {
    const TABLE_ARGS: &'static str = "id UNIQUE NOT NULL";
    const TABLE_NAME: &'static str = "exclude";

    fn import(db: &DataBase, data: &str) {
        let data = crate::utils::parse(data);
        for id in data {
            Self::add_exclude(db, id)
        }
    }

    fn export(db: &DataBase) -> String {
        crate::utils::to_string(Self::get_excludes(db).into_iter())
    }
}
