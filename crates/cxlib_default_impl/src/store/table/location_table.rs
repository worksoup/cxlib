use crate::store::{AliasTable, DataBase, DataBaseTableTrait};
use cxlib_error::StoreError;
use cxlib_store::StorageTableCommandTrait;
use cxlib_types::Location;
use log::{debug, warn};
use std::collections::HashMap;
use std::fmt::Display;
use std::str::FromStr;

pub struct LocationTable;
pub struct LocationAndAliasesPair {
    pub course: i64,
    pub location: Location,
    pub aliases: Vec<String>,
}
impl FromStr for LocationAndAliasesPair {
    type Err = cxlib_error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let data: Vec<&str> = s.split('$').collect();

        if data.len() > 1 {
            let course = data[0].trim().parse::<i64>().unwrap_or_else(|e| {
                warn!("课程号解析失败，回退为 `-1`! 错误信息：{e}.");
                -1_i64
            });
            match Location::parse(data[1]) {
                Ok(location) => {
                    let aliases: Vec<_> = if data.len() > 2 {
                        data[2].split('/').map(|s| s.trim().to_string()).collect()
                    } else {
                        vec![]
                    };
                    Ok(LocationAndAliasesPair {
                        course,
                        location,
                        aliases,
                    })
                }
                Err(e) => Err(StoreError::ParseError(format!("位置解析出错：{e}.")))?,
            }
        } else {
            Err(StoreError::ParseError(
                "格式应为 `课程号$地址,经度,纬度,海拔$别名/...`".to_string(),
            ))?
        }
    }
}
impl Display for LocationAndAliasesPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let aliases_contents = self.aliases.join("/");
        debug!("{:?}", self.aliases);
        write!(f, "{}${}${}", self.course, self.location, aliases_contents)
    }
}
impl LocationTable {
    pub fn has_location(db: &DataBase, location_id: i64) -> bool {
        let mut query = db
            .prepare(format!(
                "SELECT count(*) FROM {} WHERE lid=?;",
                Self::TABLE_NAME
            ))
            .unwrap();
        query.bind((1, location_id)).unwrap();
        query.next().unwrap();
        query.read::<i64, _>(0).unwrap() > 0
    }
    pub fn add_location_or<O: Fn(&DataBase, i64, i64, &Location)>(
        db: &DataBase,
        location_id: i64,
        course_id: i64,
        location: &Location,
        or: O,
    ) {
        let addr = location.get_addr();
        let lat = location.get_lat();
        let lon = location.get_lon();
        let alt = location.get_alt();
        let mut query =db.prepare(format!("INSERT INTO {}(lid,courseid,addr,lat,lon,alt) values(:lid,:courseid,:addr,:lat,:lon,:alt);",Self::TABLE_NAME)).unwrap();
        query
            .bind::<&[(_, sqlite::Value)]>(
                &[
                    (":lid", location_id.into()),
                    (":courseid", course_id.into()),
                    (":addr", addr.into()),
                    (":lat", lat.into()),
                    (":lon", lon.into()),
                    (":alt", alt.into()),
                ][..],
            )
            .unwrap();
        match query.next() {
            Ok(_) => (),
            Err(_) => or(db, location_id, course_id, location),
        }
    }
    /// 添加位置，返回 LocationId.
    pub fn insert_location(db: &DataBase, course_id: i64, location: &Location) -> i64 {
        // 为指定课程添加位置。
        let mut lid = 0_i64;
        loop {
            if Self::has_location(db, lid) {
                lid += 1;
                continue;
            }
            Self::add_location_or(db, lid, course_id, location, |_, _, _, _| {});
            break;
        }
        lid
    }
    pub fn delete_location(db: &DataBase, location_id: i64) {
        db.execute(format!(
            "DELETE FROM {} WHERE lid={location_id};",
            Self::TABLE_NAME
        ))
        .unwrap();
        let aliases = AliasTable::get_aliases(db, location_id);
        for alias in aliases {
            AliasTable::delete_alias(db, &alias)
        }
    }
    /// location_id, (course_id, location)
    pub fn get_locations(db: &DataBase) -> HashMap<i64, (i64, Location)> {
        let mut query = db
            .prepare(format!("SELECT * FROM {};", Self::TABLE_NAME))
            .unwrap();
        let mut location_map = HashMap::new();
        for c in query.iter() {
            if let Ok(row) = c {
                let os_id = row.read("lid");
                let addr = row.read("addr");
                let lat = row.read("lat");
                let lon = row.read("lon");
                let alt = row.read("alt");
                let course_id = row.read("courseid");
                location_map.insert(os_id, (course_id, Location::new(addr, lon, lat, alt)));
            } else {
                warn!("位置解析行出错：{c:?}.");
            }
        }
        location_map
    }
    pub fn get_location(db: &DataBase, location_id: i64) -> (i64, Location) {
        let mut query = db
            .prepare(format!("SELECT * FROM {} WHERE lid=?;", Self::TABLE_NAME))
            .unwrap();
        query.bind((1, location_id)).unwrap();
        let c: Vec<sqlite::Row> = query
            .iter()
            .filter_map(|e| if let Ok(e) = e { Some(e) } else { None })
            .collect();
        let row = &c[0];
        let addr = row.read("addr");
        let lat = row.read("lat");
        let lon = row.read("lon");
        let alt = row.read("alt");
        let course_id = row.read("courseid");
        (course_id, Location::new(addr, lon, lat, alt))
    }
    pub fn get_location_by_alias(db: &DataBase, alias: &str) -> Option<Location> {
        AliasTable::get_location_id(db, alias).map(|id| Self::get_location(db, id).1)
    }
    pub fn get_location_map_by_course(db: &DataBase, course_id: i64) -> HashMap<i64, Location> {
        let mut query = db
            .prepare(format!(
                "SELECT * FROM {} WHERE courseid=?;",
                Self::TABLE_NAME
            ))
            .unwrap();
        query.bind((1, course_id)).unwrap();
        let mut location_map = HashMap::new();
        for c in query.iter() {
            if let Ok(row) = c {
                let location_id = row.read("lid");
                let addr = row.read("addr");
                let lon = row.read("lon");
                let lat = row.read("lat");
                let alt = row.read("alt");
                location_map.insert(location_id, Location::new(addr, lon, lat, alt));
            } else {
                warn!("位置解析行出错：{c:?}.");
            }
        }
        location_map
    }
    pub fn get_location_list_by_course(db: &DataBase, course_id: i64) -> Vec<Location> {
        let mut query = db
            .prepare(format!(
                "SELECT * FROM {} WHERE courseid=?;",
                Self::TABLE_NAME
            ))
            .unwrap();
        query.bind((1, course_id)).unwrap();
        let mut location_list = Vec::new();
        for c in query.iter() {
            if let Ok(row) = c {
                let addr = row.read("addr");
                let lat = row.read("lat");
                let lon = row.read("lon");
                let alt = row.read("alt");
                location_list.push(Location::new(addr, lon, lat, alt));
            } else {
                warn!("位置解析行出错：{c:?}.");
            }
        }
        location_list
    }
}
impl StorageTableCommandTrait<DataBase> for LocationTable {
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
impl DataBaseTableTrait for LocationTable {
    const TABLE_ARGS: &'static str = "lid INTEGER UNIQUE NOT NULL,courseid INTEGER NOT NULL,addr TEXT NOT NULL,lon TEXT NOT NULL,lat TEXT NOT NULL,alt TEXT NOT NULL";
    const TABLE_NAME: &'static str = "location";

    fn import(db: &DataBase, data: &str) {
        let data = crate::utils::parse::<cxlib_error::Error, LocationAndAliasesPair>(data);
        for LocationAndAliasesPair {
            course,
            location,
            aliases,
        } in data
        {
            let location_id = Self::insert_location(db, course, &location);
            for alias in aliases {
                if !alias.is_empty() {
                    AliasTable::add_alias_or(db, &alias, location_id, AliasTable::update_alias)
                }
            }
        }
    }

    fn export(db: &DataBase) -> String {
        let data = Self::get_locations(db)
            .into_iter()
            .map(|(location_id, (course, location))| {
                let aliases = AliasTable::get_aliases(db, location_id);
                LocationAndAliasesPair {
                    course,
                    location,
                    aliases,
                }
            });
        crate::utils::to_string(data)
    }
}
