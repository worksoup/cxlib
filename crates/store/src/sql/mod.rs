pub mod account_table;
pub mod alias_table;
pub mod course_table;
pub mod data_base;
pub mod location_table;

use crate::sql::data_base::DataBase;

pub trait DataBaseTableTrait<'a> {
    const TABLE_ARGS: &'static str;
    const TABLE_NAME: &'static str;
    fn from_ref(db: &'a DataBase) -> Self;
    fn create(db: &DataBase) {
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
        println!("已删除数据表 {}。", Self::TABLE_NAME);
    }
}
