use cxsign_error::Error;
use cxsign_sign::{SignResult, SignTrait};
use cxsign_types::Location;
use cxsign_user::Session;
use std::collections::HashMap;

pub trait SignnerTrait<T: SignTrait> {
    type ExtData<'e>;
    fn sign<'a, Sessions: Iterator<Item = &'a Session> + Clone>(
        &mut self,
        sign: &mut T,
        sessions: Sessions,
    ) -> Result<HashMap<&'a Session, SignResult>, Error>;
    fn sign_single(
        sign: &mut T,
        session: &Session,
        extra_data: Self::ExtData<'_>,
    ) -> Result<SignResult, Error>;
}


pub trait LocationInfoGetterTrait<Sign: SignTrait> {
    fn get_preset_location(&self, sign: &Sign) -> Option<Location>;
    fn map_location_str(&self, location_str: &str) -> Option<Location>;
    fn get_location_or(
        &self,
        location_str: &Option<String>,
        preset_location: Option<Location>,
    ) -> Option<Location> {
        self.get_location_or_else(location_str, || preset_location)
    }

    fn get_location_or_else<GetPresetLocation: FnOnce() -> Option<Location>>(
        &self,
        location_str: &Option<String>,
        get_preset_location: GetPresetLocation,
    ) -> Option<Location> {
        location_str.as_ref().and_then(|location_str| {
            self.map_location_str(location_str).or_else(|| {
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
    fn get_fallback_location(&self, sign: &Sign) -> Option<Location>;
    fn get_locations(&self, sign: &Sign, location_str: &Option<String>) -> Location {
        self.get_location_or_else(location_str, || self.get_preset_location(sign))
            .or_else(|| self.get_fallback_location(sign))
            .unwrap_or_else(Location::get_none_location)
    }
}
