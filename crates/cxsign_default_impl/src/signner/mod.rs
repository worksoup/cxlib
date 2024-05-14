mod impls;
pub mod utils;

use cxsign_activity::sign::{LocationSign, SignTrait};
use cxsign_store::{DataBase, DataBaseTableTrait};
use cxsign_types::{Location, LocationTable};
pub use impls::*;
use utils::location_str_to_location;

pub trait LocationInfoGetterTrait {
    fn get_preset_location(&self, sign: &LocationSign) -> Option<Location>;
    fn get_location_or(
        &self,
        location_str: &Option<String>,
        preset_location: Option<Location>,
    ) -> Option<Location>;
    fn get_location_or_else<GetPresetLocation: FnOnce() -> Option<Location>>(
        &self,
        location_str: &Option<String>,
        get_preset_location: GetPresetLocation,
    ) -> Option<Location>;
    fn get_fallback_location(&self, sign: &LocationSign) -> Option<Location>;
    fn get_locations(&self, sign: &LocationSign, location_str: &Option<String>) -> Location;
}

pub struct DefaultLocationInfoGetter<'a>(&'a DataBase);
impl<'a> DefaultLocationInfoGetter<'a> {
    pub fn new(db: &'a DataBase) -> Self {
        Self(db)
    }
}
impl<'a> From<&'a DataBase> for DefaultLocationInfoGetter<'a> {
    fn from(db: &'a DataBase) -> Self {
        Self::new(db)
    }
}

impl LocationInfoGetterTrait for DefaultLocationInfoGetter<'_> {
    fn get_preset_location(&self, sign: &LocationSign) -> Option<Location> {
        sign.get_preset_location(None)
    }

    fn get_location_or(
        &self,
        location_str: &Option<String>,
        preset_location: Option<Location>,
    ) -> Option<Location> {
        match location_str_to_location(self.0, location_str) {
            Ok(location) => Some(location),
            Err(location_str) => preset_location.map(|mut l| {
                if location_str.is_empty() {
                    l
                } else {
                    l.set_addr(&location_str);
                    l
                }
            }),
        }
    }

    fn get_location_or_else<GetPresetLocation: FnOnce() -> Option<Location>>(
        &self,
        location_str: &Option<String>,
        get_preset_location: GetPresetLocation,
    ) -> Option<Location> {
        match location_str_to_location(self.0, location_str) {
            Ok(location) => Some(location),
            Err(location_str) => get_preset_location().map(|mut l| {
                if location_str.is_empty() {
                    l
                } else {
                    l.set_addr(&location_str);
                    l
                }
            }),
        }
    }
    fn get_fallback_location(&self, sign: &LocationSign) -> Option<Location> {
        let table = LocationTable::from_ref(self.0);
        table
            .get_location_list_by_course(sign.as_inner().course.get_id())
            .pop()
            .or_else(|| table.get_location_list_by_course(-1).pop())
    }
    fn get_locations(&self, sign: &LocationSign, location_str: &Option<String>) -> Location {
        self.get_location_or_else(location_str, || self.get_preset_location(sign))
            .or_else(|| self.get_fallback_location(sign))
            .unwrap_or_else(Location::get_none_location)
    }
}
