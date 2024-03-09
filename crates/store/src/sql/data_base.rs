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

// 位置
impl DataBase {
    const CREATE_POS_SQL: &'static str ="CREATE TABLE location(lid INTEGER UNIQUE NOT NULL,courseid INTEGER NOT NULL,addr TEXT NOT NULL,lon TEXT NOT NULL,lat TEXT NOT NULL,alt TEXT NOT NULL);";

    fn has_table_location(&self) -> bool {
        let mut query = self
            .connection
            .prepare("SELECT count(*) FROM sqlite_master WHERE type='table' AND name='location';")
            .unwrap();
        query.next().unwrap();
        query.read::<i64, _>(0).unwrap() == 1
    }
    // pub fn 删除所有位置(&self) {
    //     self.connection.execute("DELETE FROM location;").unwrap();
    //     self.删除所有别名();
    // }
    fn create_table_location(&self) {
        if !self.has_table_location() {
            self.connection.execute(Self::CREATE_POS_SQL).unwrap();
        }
    }
}

// alias
impl DataBase {
    const CREATE_ALIAS_SQL: &'static str =
        "CREATE TABLE alias (name CHAR (50) UNIQUE NOT NULL,lid INTEGER NOT NULL);";

    fn has_table_alias(&self) -> bool {
        let mut query = self
            .connection
            .prepare("SELECT count(*) FROM sqlite_master WHERE type='table' AND name='alias';")
            .unwrap();
        query.next().unwrap();
        query.read::<i64, _>(0).unwrap() == 1
    }

    fn create_table_alias(&self) {
        if !self.has_table_alias() {
            self.connection.execute(Self::CREATE_ALIAS_SQL).unwrap();
        }
    }
}

// account
impl DataBase {
    const CREATE_ACCOUNT_SQL: &'static str =
        "CREATE TABLE account (uname CHAR (50) UNIQUE NOT NULL,pwd TEXT NOT NULL,name TEXT NOT NULL);";

    fn has_table_account(&self) -> bool {
        let mut query = self
            .connection
            .prepare("SELECT count(*) FROM sqlite_master WHERE type='table' AND name='account';")
            .unwrap();
        query.next().unwrap();
        query.read::<i64, _>(0).unwrap() == 1
    }

    fn create_table_account(&self) {
        if !self.has_table_account() {
            self.connection.execute(Self::CREATE_ACCOUNT_SQL).unwrap();
        }
    }
}

// course
impl DataBase {
    const CREATE_COURSE_SQL: &'static str ="CREATE TABLE course (id INTEGER UNIQUE NOT NULL,clazzid INTEGER NOT NULL,name TEXT NOT NULL,teacher TEXT NOT NULL,image TEXT NOT NULL);";

    fn has_table_course(&self) -> bool {
        let mut query = self
            .connection
            .prepare("SELECT count(*) FROM sqlite_master WHERE type='table' AND name='course';")
            .unwrap();
        query.next().unwrap();
        query.read::<i64, _>(0).unwrap() == 1
    }
    pub fn delete_all_course(&self) {
        let mut query = self.connection.prepare("DELETE FROM course;").unwrap();
        query.next().unwrap();
        println!("已删除旧的课程信息。");
    }
    fn create_table_course(&self) {
        if !self.has_table_course() {
            self.connection.execute(Self::CREATE_COURSE_SQL).unwrap();
        }
    }
}
