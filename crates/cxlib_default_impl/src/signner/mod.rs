mod impls;
pub mod utils;

use crate::sign::LocationSign;
use crate::store::{DataBase, LocationTable};
use cxlib_sign::SignTrait;
use cxlib_types::Location;
pub use impls::*;

pub trait LocationInfoGetterTrait {
    fn location_str_to_location(&self, location_str: &str) -> Option<Location>;
    fn get_fallback_location(&self, sign: &LocationSign) -> Option<Location>;
    fn get_location_or_else<GetPresetLocation: FnOnce() -> Option<Location>>(
        &self,
        location_str: &Option<String>,
        get_preset_location: GetPresetLocation,
    ) -> Option<Location> {
        location_str.as_ref().and_then(|location_str| {
            self.location_str_to_location(location_str).or_else(|| {
                get_preset_location().map(|mut l| {
                    if location_str.is_empty() {
                        l
                    } else {
                        l.set_addr(location_str);
                        l
                    }
                })
            })
        })
    }
    fn get_preset_location(&self, sign: &LocationSign) -> Option<Location> {
        sign.get_preset_location()
    }
    fn get_location_or(
        &self,
        location_str: &Option<String>,
        preset_location: Option<Location>,
    ) -> Option<Location> {
        self.get_location_or_else(location_str, || preset_location)
    }
    fn get_locations(
        &self,
        sign: &LocationSign,
        location_str: &Option<String>,
    ) -> Option<Location> {
        self.get_location_or_else(location_str, || self.get_preset_location(sign))
            .or_else(|| self.get_fallback_location(sign))
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
    fn location_str_to_location(&self, location_str: &str) -> Option<Location> {
        let location_str = location_str.trim();
        location_str
            .parse()
            .ok()
            .or_else(|| LocationTable::get_location_by_alias(self.0, location_str))
            .or_else(|| {
                location_str
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
