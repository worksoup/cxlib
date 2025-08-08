mod internal_data;

use crate::{
    AliasTable, BinCode, CourseTable, ImportExportTrait, NormalTableTrait, StoreError,
    TableDefinitionTrait, database_guard::DatabaseGuard,
    default_impl::table::location_table::internal_data::LocationAndAliasesPairInternal,
};
use cxlib_error_utils::{CxlibResultUtils, MaybeFatalError};
use cxlib_internal::types::{__private::UnhandledGeoaddr, Course, Geolocation};
use log::warn;
use redb::{ReadableTable, WriteTransaction};
use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
};

pub struct LocationTable;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, bincode::Encode, bincode::Decode)]
pub struct LocationTableData {
    unhandled_addr: String,
    courses: Vec<Course>,
}
impl LocationTable {
    pub fn has_location(
        table: &impl ReadableTable<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
        location: impl Borrow<Geolocation>,
    ) -> Result<bool, StoreError> {
        Self::contains_key(table, location)
    }

    pub fn add_location(
        w_cxt: &WriteTransaction,
        location: impl Borrow<Geolocation>,
        unhandled_addr: impl Borrow<String>,
        courses: impl Borrow<Vec<Course>>,
    ) -> Result<(), StoreError> {
        let mut location_table = Self::write(w_cxt)?;
        let data = LocationTableData {
            unhandled_addr: unhandled_addr.borrow().to_owned(),
            courses: courses.borrow().to_owned(),
        };
        location_table.insert(location.borrow(), data)?;
        drop(location_table);
        let mut course_table = CourseTable::write(w_cxt)?;
        for course in courses.borrow() {
            match CourseTable::contains_key(&course_table, course).and_then(|contains| {
                if contains {
                    let Some(mut locations) = CourseTable::get_course(&course_table, course)?
                    else {
                        unreachable!()
                    };
                    let addr = UnhandledGeoaddr::new(
                        unhandled_addr.borrow().to_owned(),
                        location.borrow().clone(),
                    );
                    locations.1.get_locations_mut().push(addr);
                    Ok(locations)
                } else {
                    Err(StoreError::LogicError(
                        "课程未添加到数据库中，无法为其指定位置。".to_owned(),
                    ))
                }
            }) {
                Ok(course_data) => {
                    CourseTable::insert_course(&mut course_table, course, course_data)?;
                }
                Err(e) => {
                    if e.is_fatal() {
                        Err(e)?;
                    } else {
                        warn!("`{e}`.");
                        continue;
                    }
                }
            }
        }
        Ok(())
    }
    /// 添加位置，返回 LocationId.
    pub fn insert_location(
        w_cxt: &WriteTransaction,
        location: impl Borrow<Geolocation>,
        unhandled_addr: impl Borrow<String>,
        course: impl Borrow<Vec<Course>>,
        aliases: &[String],
    ) -> Result<(), StoreError> {
        Self::add_location(w_cxt, location.borrow(), unhandled_addr.borrow(), course)?;
        let mut alias_table = AliasTable::write(w_cxt)?;
        for alias in aliases {
            let addr = UnhandledGeoaddr::new(
                unhandled_addr.borrow().to_owned(),
                location.borrow().clone(),
            );
            let e = AliasTable::add_alias(&mut alias_table, alias, addr);
            if let Err(e) = e
                && e.is_fatal()
            {
                return Err(e);
            }
        }
        Ok(())
    }
    pub fn delete_location(
        w_cxt: &WriteTransaction,
        location: &Geolocation,
    ) -> Result<(), StoreError> {
        let mut removed: Vec<&Geolocation> = Vec::new();
        let mut location_table = Self::write(w_cxt)?;
        if let Some(_course) = location_table.remove(location).log_unwrap() {
            removed.push(location);
        }
        drop(location_table);
        let alias_table = AliasTable::write(w_cxt)?;
        let mut aliases_being_removed = HashSet::new();
        for removed in removed {
            let aliases = match AliasTable::get_aliases(&alias_table, removed) {
                Ok(aliases) => aliases,
                Err(e) => {
                    if e.is_fatal() {
                        return Err(e);
                    } else {
                        vec![]
                    }
                }
            };
            aliases_being_removed.extend(aliases);
        }
        let mut alias_table = alias_table;
        for alias in aliases_being_removed {
            if let Err(e) = AliasTable::delete_alias(&mut alias_table, &alias)
                && e.is_fatal()
            {
                return Err(e);
            }
        }
        drop(alias_table);
        Ok(())
    }
    /// location_id, (course_id, location)
    pub fn get_locations(
        table: &impl ReadableTable<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
    ) -> Result<HashMap<Geolocation, (String, Vec<Course>)>, StoreError> {
        Ok(table
            .iter()?
            .flatten()
            .map(|(k, v)| {
                let v = v.value();
                (k.value(), (v.unhandled_addr, v.courses))
            })
            .collect())
    }
    pub fn get_geoaddrs(
        table: &impl ReadableTable<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
    ) -> Result<HashMap<UnhandledGeoaddr, Vec<Course>>, StoreError> {
        Ok(table
            .iter()?
            .flatten()
            .map(|(k, v)| {
                let v = v.value();
                (
                    UnhandledGeoaddr::new(v.unhandled_addr, k.value()),
                    v.courses,
                )
            })
            .collect())
    }
    pub fn get_place_name(
        table: &impl ReadableTable<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
        location: impl Borrow<Geolocation>,
    ) -> Result<String, StoreError> {
        table
            .get(location)?
            .map(|a| a.value().unhandled_addr)
            .ok_or_else(|| {
                StoreError::UnexpectedNone("位置对应地名不存在，请检查是否存在 Bug.".to_owned())
            })
    }
    pub fn get_courses(
        table: &impl ReadableTable<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
        location: impl Borrow<Geolocation>,
    ) -> Result<Vec<Course>, StoreError> {
        Ok(table
            .get(location)?
            .map(|a| a.value().courses)
            .unwrap_or_default())
    }
    // pub fn get_location_by_alias(db: &Database, alias: &str) -> Option<UnhandledLocation> {
    //     AliasTable::get_location_id(db, alias).map(|id| Self::get_location(db, id).1)
    // }
    // pub fn get_location_map_by_course(db: &Database, course_id: i64) -> HashMap<i64, UnhandledLocation> {
    //     let mut query = db
    //         .prepare(format!(
    //             "SELECT * FROM {} WHERE courseid=?;",
    //             Self::TABLE_NAME
    //         ))
    //         .unwrap();
    //     query.bind((1, course_id)).unwrap();
    //     let mut location_map = HashMap::new();
    //     for c in query.iter() {
    //         if let Ok(row) = c {
    //             let location_id = row.read("lid");
    //             let addr = row.read("addr");
    //             let lon = row.read("lon");
    //             let lat = row.read("lat");
    //             let alt = row.read("alt");
    //             location_map.insert(location_id, UnhandledLocation::new(addr, lon, lat, alt));
    //         } else {
    //             warn!("位置解析行出错：{c:?}.");
    //         }
    //     }
    //     location_map
    // }
    //     pub fn get_location_list_by_course(db: &Database, course_id: i64) -> Vec<UnhandledLocation> {
    //         let mut query = db
    //             .prepare(format!(
    //                 "SELECT * FROM {} WHERE courseid=?;",
    //                 Self::TABLE_NAME
    //             ))
    //             .unwrap();
    //         query.bind((1, course_id)).unwrap();
    //         let mut location_list = Vec::new();
    //         for c in query.iter() {
    //             if let Ok(row) = c {
    //                 let addr = row.read("addr");
    //                 let lat = row.read("lat");
    //                 let lon = row.read("lon");
    //                 let alt = row.read("alt");
    //                 location_list.push(UnhandledLocation::new(addr, lon, lat, alt));
    //             } else {
    //                 warn!("位置解析行出错：{c:?}.");
    //             }
    //         }
    //         location_list
    //     }
}
impl NormalTableTrait for LocationTable {}
impl TableDefinitionTrait for LocationTable {
    type Key = BinCode<Geolocation>;
    type Value = BinCode<LocationTableData>;
    type Context<'cxt> = ();
    const NAME: &'static str = "location";
}
impl ImportExportTrait for LocationTable {
    fn import_text<'cxt, Cxt: Borrow<Self::Context<'cxt>>>(
        db: &mut DatabaseGuard,
        cxt: Cxt,
        content: &str,
    ) {
        let data: Vec<LocationAndAliasesPairInternal> =
            crate::default_impl::parse_lines::<_, _>(content, cxt);
        db.write_once(|w_cxt| {
            for location_and_aliases_pair_internal in data {
                let LocationAndAliasesPairInternal {
                    unhandled_geoaddr,
                    courses,
                    aliases,
                } = location_and_aliases_pair_internal;
                _ = Self::insert_location(
                    w_cxt,
                    unhandled_geoaddr.geolocation(),
                    unhandled_geoaddr.unhandled_place_name(),
                    courses,
                    &aliases,
                );
            }
            Ok::<_, StoreError>(())
        })
        .log_unwrap();
    }

    fn export_text(db: &mut DatabaseGuard) -> String {
        db.read_once(|r_cxt| {
            let mut data = HashSet::new();

            let alias_table = AliasTable::read(r_cxt).log_unwrap();
            let location_table = LocationTable::read(r_cxt).log_unwrap();
            for (location, (unhandled_addr, courses)) in
                Self::get_locations(&location_table).log_unwrap()
            {
                match AliasTable::get_aliases(&alias_table, &location) {
                    Ok(aliases) => {
                        let unhandled_addr = UnhandledGeoaddr::new(unhandled_addr, location);
                        let datum =
                            LocationAndAliasesPairInternal::new(unhandled_addr, courses, aliases);
                        data.insert(datum);
                    }
                    Err(e) => {
                        if e.is_fatal() {
                            warn!("{e}, 数据已终止导入。");
                            return Err(e);
                        }
                        warn!("{e}");
                    }
                };
            }
            Ok(crate::default_impl::to_string_lines(data))
        })
        .log_unwrap()
    }
}
