use crate::store::{DataBase, DataBaseTableTrait};
use cxsign_store::StorageTableCommandTrait;
use log::warn;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::str::FromStr;

pub struct KVConfigTable;
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KVPair {
    pub key: String,
    pub value: String,
}
impl From<(String, String)> for KVPair {
    fn from((key, value): (String, String)) -> Self {
        KVPair { key, value }
    }
}
impl Display for KVPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{}", self.key, self.value)
    }
}
impl FromStr for KVPair {
    type Err = cxsign_error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();
        if s.len() < 2 {
            Err(cxsign_error::Error::ParseError(
                "键值表解析出错！格式为 `key, config`.".to_string(),
            ))
        } else {
            let key = s[0].to_string();
            let value = s[1].to_string();
            Ok(Self { key, value })
        }
    }
}
impl KVConfigTable {
    pub fn keys(db: &DataBase) -> HashSet<String> {
        let mut query = db
            .prepare(format!("SELECT * FROM {};", Self::TABLE_NAME))
            .unwrap();
        let mut s = HashSet::new();
        for c in query.iter() {
            if let Ok(row) = c {
                let key: &str = row.read("key");
                s.insert(key.into());
            } else {
                warn!("kv_config 解析行出错：{c:?}.");
            }
        }
        s
    }
    pub fn values(db: &DataBase) -> Vec<String> {
        let mut query = db
            .prepare(format!("SELECT * FROM {};", Self::TABLE_NAME))
            .unwrap();
        let mut s = Vec::new();
        for c in query.iter() {
            if let Ok(row) = c {
                let value: &str = row.read("value");
                s.push(value.into());
            } else {
                warn!("kv_config 解析行出错：{c:?}.");
            }
        }
        s
    }
    pub fn get_as_map_by_keys_str(db: &DataBase, keys: &str) -> HashMap<String, String> {
        let str_list = keys.split(',').map(|a| a.trim()).collect::<Vec<&str>>();
        let mut s = HashMap::new();
        for key in str_list {
            if let Some(session) = Self::get_by_key(db, key) {
                s.insert(key.to_string(), session);
            }
        }
        s
    }
    pub fn get_by_key(db: &DataBase, key: &str) -> Option<String> {
        let mut query = db
            .prepare(format!("SELECT * FROM {} WHERE key=?;", Self::TABLE_NAME))
            .unwrap();
        query.bind((1, key)).unwrap();
        for c in query.iter() {
            if let Ok(row) = c {
                let value: &str = row.read("value");
                return Some(value.into());
            } else {
                warn!("kv_config 解析行出错：{c:?}.");
            }
        }
        None
    }
    pub fn get_as_map(db: &DataBase) -> HashMap<String, String> {
        let mut query = db
            .prepare(format!("SELECT * FROM {};", Self::TABLE_NAME))
            .unwrap();
        let mut s = HashMap::new();
        for c in query.iter() {
            if let Ok(row) = c {
                let key: &str = row.read("key");
                let value: &str = row.read("value");
                s.insert(key.into(), value.into());
            } else {
                warn!("kv_config 解析行出错：{c:?}.");
            }
        }
        s
    }
    pub fn has_key(db: &DataBase, key: &str) -> bool {
        let mut query = db
            .prepare(format!(
                "SELECT count(*) FROM {} WHERE key=?;",
                Self::TABLE_NAME
            ))
            .unwrap();
        query.bind((1, key)).unwrap();
        query.next().unwrap();
        query.read::<i64, _>(0).unwrap() > 0
    }

    pub fn remove(db: &DataBase, key: &str) {
        if Self::has_key(db, key) {
            let mut query = db
                .prepare(format!("DELETE FROM {} WHERE key=?;", Self::TABLE_NAME))
                .unwrap();
            query.bind((1, key)).unwrap();
            query.next().unwrap();
        }
    }

    pub fn insert_or<O: Fn(&DataBase, &str, &str)>(db: &DataBase, key: &str, value: &str, or: O) {
        let mut query = db
            .prepare(format!(
                "INSERT INTO {}(key,value) values(:key,:value);",
                Self::TABLE_NAME
            ))
            .unwrap();
        query
            .bind::<&[(_, sqlite::Value)]>(&[(":value", value.into()), (":key", key.into())][..])
            .unwrap();
        match query.next() {
            Ok(_) => (),
            Err(_) => or(db, key, value),
        };
    }

    pub fn update(db: &DataBase, key: &str, value: &str) {
        let mut query = db
            .prepare(format!(
                "UPDATE {} SET value=:value WHERE key=:key;",
                Self::TABLE_NAME
            ))
            .unwrap();
        query
            .bind::<&[(_, sqlite::Value)]>(&[(":key", key.into()), (":value", value.into())][..])
            .unwrap();
        query.next().unwrap();
    }

    pub fn get_pairs(db: &DataBase) -> HashSet<KVPair> {
        Self::get_as_map(db)
            .into_iter()
            .map(|pair| pair.into())
            .collect()
    }

    pub fn get_pair(db: &DataBase, key: &str) -> Option<KVPair> {
        Self::get_by_key(db, key).map(|value| (key.to_owned(), value).into())
    }
}

impl StorageTableCommandTrait<DataBase> for KVConfigTable {
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
impl DataBaseTableTrait for KVConfigTable {
    const TABLE_ARGS: &'static str = "key CHAR (50) UNIQUE NOT NULL,value TEXT NOT NULL";
    const TABLE_NAME: &'static str = "kv_config";

    fn import(db: &DataBase, data: &str) {
        db.add_table::<Self>();
        let data = crate::utils::parse::<cxsign_error::Error, KVPair>(data);
        for KVPair { key, value } in data {
            Self::insert_or(db, &key, &value, Self::update)
        }
    }

    fn export(db: &DataBase) -> String {
        crate::utils::to_string(Self::get_pairs(db).into_iter())
    }
}
