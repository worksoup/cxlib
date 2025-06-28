mod impls;
pub mod utils;

pub use impls::*;

use crate::sign::LocationSign;
use cxlib_types::{__private::UnhandledGeoaddr, Geoaddr, LocationPreprocessorTrait};

pub trait LocationInfoGetterTrait {
    fn get_location_by_location_str(
        &self,
        location_str: &str,
        preprocessor: &impl LocationPreprocessorTrait,
    ) -> Option<Geoaddr>;
    fn get_fallback_location<SignProtocol>(
        &self,
        sign: &LocationSign<SignProtocol>,
        preprocessor: &impl LocationPreprocessorTrait,
    ) -> Option<Geoaddr>;
    fn get_locations<SignProtocol>(
        &self,
        sign: &LocationSign<SignProtocol>,
        location_str: &Option<String>,
        location_preprocessor: &impl LocationPreprocessorTrait,
    ) -> Vec<Geoaddr> {
        let mut locations = Vec::new();
        // 该位置保证能够签到成功。
        let l2 = sign.get_preset_location(location_preprocessor);
        if let Some(location_str) = location_str {
            let location_str = location_str.trim();
            let l1 = location_str
                .parse::<UnhandledGeoaddr>()
                .ok()
                .map(|l| l.to_location(location_preprocessor));
            if let Some(l1) = l1 {
                locations.push(l1);
            } else if let Some(mut l2) = l2 {
                l2.set_addr(location_str.to_owned());
                locations.push(l2);
                return locations;
            }
        } else if let Some(l2) = l2 {
            locations.push(l2);
            return locations;
        }
        let l3 = self.get_fallback_location(sign, location_preprocessor);
        if let Some(l3) = l3 {
            locations.push(l3);
        }
        locations
    }
}
