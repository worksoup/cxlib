use crate::store::{DataBase, DataBaseTableTrait};
use cxlib_store::StorageTableCommandTrait;
use log::warn;

pub struct AliasTable;

impl AliasTable {
    pub fn has_alias(db: &DataBase, alias: &str) -> bool {
        let mut query = db
            .prepare(format!(
                "SELECT count(*) FROM {} WHERE name=?;",
                Self::TABLE_NAME
            ))
            .unwrap();
        query.bind((1, alias)).unwrap();
        query.next().unwrap();
        query.read::<i64, _>(0).unwrap() > 0
    }

    pub fn delete_alias(db: &DataBase, alias: &str) {
        let mut query = db
            .prepare(format!("DELETE FROM {} WHERE name=?;", Self::TABLE_NAME))
            .unwrap();
        query.bind((1, alias)).unwrap();
        query.next().unwrap();
    }

    pub fn add_alias_or<O: Fn(&DataBase, &str, i64)>(
        db: &DataBase,
        alias: &str,
        location_id: i64,
        or: O,
    ) {
        let mut query = db
            .prepare(format!(
                "INSERT INTO {}(name,lid) values(:name,:lid);",
                Self::TABLE_NAME
            ))
            .unwrap();
        query
            .bind::<&[(_, sqlite::Value)]>(
                &[(":name", alias.into()), (":lid", location_id.into())][..],
            )
            .unwrap();
        match query.next() {
            Ok(_) => (),
            Err(_) => or(db, alias, location_id),
        };
    }
    pub fn update_alias(db: &DataBase, alias: &str, location_id: i64) {
        let mut query = db
            .prepare(format!(
                "UPDATE {} SET name=:name,lid=:lid WHERE name=:name;",
                Self::TABLE_NAME
            ))
            .unwrap();
        query
            .bind::<&[(_, sqlite::Value)]>(
                &[(":name", alias.into()), (":lid", location_id.into())][..],
            )
            .unwrap();
        query.next().unwrap();
    }
    pub fn get_aliases(db: &DataBase, location_id: i64) -> Vec<String> {
        let mut query = db
            .prepare(format!("SELECT * FROM {} WHERE lid=?;", Self::TABLE_NAME))
            .unwrap();
        query.bind((1, location_id)).unwrap();
        let mut aliases = Vec::new();
        for c in query.iter() {
            if let Ok(row) = c {
                let name: &str = row.read("name");
                aliases.push(name.to_owned());
            } else {
                warn!("位置解析行出错：{c:?}.");
            }
        }
        aliases
    }

    pub fn get_location_id(db: &DataBase, alias: &str) -> Option<i64> {
        if Self::has_alias(db, alias) {
            let mut query = db
                .prepare(format!("SELECT * FROM {} WHERE name=?;", Self::TABLE_NAME))
                .unwrap();
            query.bind((1, alias)).unwrap();
            let c: Vec<sqlite::Row> = query
                .iter()
                .filter_map(|e| if let Ok(e) = e { Some(e) } else { None })
                .collect();
            let row = &c[0];
            let location_id: i64 = row.read("lid");
            Some(location_id)
        } else {
            None
        }
    }
}

impl StorageTableCommandTrait<DataBase> for AliasTable {
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

impl DataBaseTableTrait for AliasTable {
    const TABLE_ARGS: &'static str = "name CHAR (50) UNIQUE NOT NULL,lid INTEGER NOT NULL";
    const TABLE_NAME: &'static str = "alias";
}
