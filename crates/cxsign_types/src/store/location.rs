use crate::location::Location;
use cxsign_store::{AliasTable, DataBase, DataBaseTableTrait};
use log::{debug, warn};
use std::collections::HashMap;
use std::fmt::Display;
use std::ops::Deref;
use std::str::FromStr;

pub struct LocationTable<'a> {
    db: &'a DataBase,
}
pub struct LocationAndAliasesPair {
    pub course: i64,
    pub location: Location,
    pub aliases: Vec<String>,
}
impl FromStr for LocationAndAliasesPair {
    type Err = cxsign_error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let data: Vec<&str> = s.split('$').collect();

        if data.len() > 1 {
            let course = match data[0].trim().parse::<i64>() {
                Ok(course_id) => course_id,
                Err(e) => {
                    warn!("课程号解析失败，回退为 `-1`! 错误信息：{e}.");
                    -1_i64
                }
            };
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
                Err(e) => Err(cxsign_error::Error::ParseError(format!(
                    "位置解析出错：{e}."
                ))),
            }
        } else {
            Err(cxsign_error::Error::ParseError(
                "格式应为 `课程号$地址,经度,纬度,海拔$别名/...`".to_string(),
            ))
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
impl<'a> LocationTable<'a> {
    pub fn has_location(&self, location_id: i64) -> bool {
        let mut query = self
            .db
            .prepare(format!(
                "SELECT count(*) FROM {} WHERE lid=?;",
                Self::TABLE_NAME
            ))
            .unwrap();
        query.bind((1, location_id)).unwrap();
        query.next().unwrap();
        query.read::<i64, _>(0).unwrap() > 0
    }
    pub fn add_location_or<O: Fn(&Self, i64, i64, &Location)>(
        &self,
        location_id: i64,
        course_id: i64,
        location: &Location,
        or: O,
    ) {
        let addr = location.get_addr();
        let lat = location.get_lat();
        let lon = location.get_lon();
        let alt = location.get_alt();
        let mut query =self.db.prepare(format!("INSERT INTO {}(lid,courseid,addr,lat,lon,alt) values(:lid,:courseid,:addr,:lat,:lon,:alt);",Self::TABLE_NAME)).unwrap();
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
            Err(_) => or(self, location_id, course_id, location),
        }
    }
    /// 添加位置，返回 LocationId.
    pub fn insert_location(&self, course_id: i64, location: &Location) -> i64 {
        // 为指定课程添加位置。
        let mut lid = 0_i64;
        loop {
            if self.has_location(lid) {
                lid += 1;
                continue;
            }
            self.add_location_or(lid, course_id, location, |_, _, _, _| {});
            break;
        }
        lid
    }
    pub fn delete_location(&self, location_id: i64) {
        self.db
            .execute(format!(
                "DELETE FROM {} WHERE lid={location_id};",
                Self::TABLE_NAME
            ))
            .unwrap();
        let alias_table = AliasTable::from_ref(self.db);
        let aliases = alias_table.get_aliases(location_id);
        for alias in aliases {
            alias_table.delete_alias(&alias)
        }
    }
    /// location_id, (course_id, location)
    pub fn get_locations(&self) -> HashMap<i64, (i64, Location)> {
        let mut query = self
            .db
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
    pub fn get_location(&self, location_id: i64) -> (i64, Location) {
        let mut query = self
            .db
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
    pub fn get_location_by_alias(&self, alias: &str) -> Option<Location> {
        AliasTable::from_ref(self.db)
            .get_location_id(alias)
            .map(|id| self.get_location(id).1)
    }
    pub fn get_location_map_by_course(&self, course_id: i64) -> HashMap<i64, Location> {
        let mut query = self
            .db
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
    pub fn get_location_list_by_course(&self, course_id: i64) -> Vec<Location> {
        let mut query = self
            .db
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

impl<'a> DataBaseTableTrait<'a> for LocationTable<'a> {
    const TABLE_ARGS: &'static str = "lid INTEGER UNIQUE NOT NULL,courseid INTEGER NOT NULL,addr TEXT NOT NULL,lon TEXT NOT NULL,lat TEXT NOT NULL,alt TEXT NOT NULL";
    const TABLE_NAME: &'static str = "location";

    fn from_ref(db: &'a DataBase) -> Self {
        Self { db }
    }

    fn import(db: &'a DataBase, data: String) -> Self {
        let location_table = Self::from_ref(db);
        let alias_table = AliasTable::from_ref(db);
        let data = cxsign_store::parse::<cxsign_error::Error, LocationAndAliasesPair>(data);
        for LocationAndAliasesPair {
            course,
            location,
            aliases,
        } in data
        {
            let location_id = location_table.insert_location(course, &location);
            for alias in aliases {
                if !alias.is_empty() {
                    alias_table.add_alias_or(&alias, location_id, |t, a, l| {
                        t.update_alias(a, l);
                    })
                }
            }
        }
        location_table
    }

    fn export(self) -> String {
        let alias_table = AliasTable::from_ref(&self);
        let data = self
            .get_locations()
            .into_iter()
            .map(|(location_id, (course, location))| {
                let aliases = alias_table.get_aliases(location_id);
                LocationAndAliasesPair {
                    course,
                    location,
                    aliases,
                }
            });
        cxsign_store::to_string(data)
    }
}
impl<'a> Deref for LocationTable<'a> {
    type Target = DataBase;

    fn deref(&self) -> &Self::Target {
        self.db
    }
}
