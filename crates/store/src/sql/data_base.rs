use crate::sql::DataBaseTableTrait;
use sqlite::Connection;
use std::fs::File;
use std::ops::Deref;

pub struct DataBase {
    connection: Connection,
}
impl Deref for DataBase {
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        &self.connection
    }
}
// self
impl DataBase {
    pub fn new() -> Self {
        let db_dir = base::get_database_dir();
        if db_dir.metadata().is_err() {
            File::create(db_dir.clone()).unwrap();
        }
        let connection = Connection::open(db_dir.to_str().unwrap()).unwrap();
        let db = Self { connection };
        // db.create_table_account();
        // db.create_table_course();
        // db.create_table_location();
        // db.create_table_alias();
        db
    }
    pub fn add_table<'a, T: DataBaseTableTrait<'a>>(&'a self) -> T {
        T::create(self);
        T::from_ref(self)
    }
}