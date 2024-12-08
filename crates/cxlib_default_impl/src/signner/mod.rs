mod impls;
pub mod utils;

use crate::sign::LocationSign;
use crate::store::{DataBase, LocationTable};
use cxlib_sign::SignTrait;
use cxlib_types::Location;
pub use impls::*;

pub trait LocationInfoGetterTrait {
    fn get_location_by_location_str(&self, location_str: &str) -> Option<Location>;
    fn get_fallback_location(&self, sign: &LocationSign) -> Option<Location>;
    fn get_locations(&self, sign: &LocationSign, location_str: &Option<String>) -> Vec<Location> {
        let mut locations = Vec::new();
        // 该位置保证能够签到成功。
        let l2 = sign.get_preset_location();
        if let Some(location_str) = location_str {
            let location_str = location_str.trim();
            let l1 = location_str.parse::<Location>().ok();
            if let Some(l1) = l1 {
                locations.push(l1);
            } else if let Some(mut l2) = l2 {
                l2.set_addr(location_str);
                locations.push(l2);
                return locations;
            }
        } else if let Some(l2) = l2 {
            locations.push(l2);
            return locations;
        }
        let l3 = self.get_fallback_location(sign);
        if let Some(l3) = l3 {
            locations.push(l3);
        }
        locations
    }
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
    fn get_location_by_location_str(&self, trimmed_location_str: &str) -> Option<Location> {
        LocationTable::get_location_by_alias(self.0, trimmed_location_str).or_else(|| {
            trimmed_location_str
                .parse()
                .map(|location_id| {
                    if LocationTable::has_location(self.0, location_id) {
                        let (_, location) = LocationTable::get_location(self.0, location_id);
                        Some(location)
                    } else {
                        None
                    }
                })
                .ok()
                .flatten()
        })
    }
    fn get_fallback_location(&self, sign: &LocationSign) -> Option<Location> {
        LocationTable::get_location_list_by_course(self.0, sign.as_inner().course.get_id()).pop()
    }
}
