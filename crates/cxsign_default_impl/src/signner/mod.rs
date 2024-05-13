mod impls;
pub mod utils;

use cxsign_activity::sign::{LocationSign, SignTrait};
use cxsign_store::{DataBase, DataBaseTableTrait};
use cxsign_types::{Location, LocationTable};
pub use impls::*;
use utils::location_str_to_location;

pub trait LocationInfoGetterTrait {
    fn get_locations(&self, sign: &LocationSign, location_str: &Option<String>) -> Location;
}

pub struct DefaultLocationInfoGetter<'a>(&'a DataBase);
impl<'a> DefaultLocationInfoGetter<'a> {
    pub fn new(db: &'a DataBase) -> Self {
        Self(db)
    }
}

impl LocationInfoGetterTrait for DefaultLocationInfoGetter<'_> {
    fn get_locations(&self, sign: &LocationSign, location_str: &Option<String>) -> Location {
        match location_str_to_location(self.0, location_str) {
            Ok(location) => location,
            Err(location_str) => {
                if !location_str.is_empty() {
                    if let Some(location) = sign.get_preset_location(Some(&location_str)) {
                        return location;
                    }
                } else if let Some(location) = sign.get_preset_location(None) {
                    return location;
                }

                let table = LocationTable::from_ref(self.0);
                if let Some(location) = table
                    .get_location_list_by_course(sign.as_inner().course.get_id())
                    .first()
                {
                    location.clone()
                } else if let Some(location) = table.get_location_list_by_course(-1).first() {
                    location.clone()
                } else {
                    Location::get_none_location()
                }
            }
        }
    }
}
