use crate::{sign::LocationSign, signner::LocationInfoGetterTrait};
use cxlib_sign::{SignError, SignResult, SignnerTrait};
use cxlib_types::Location;
use cxlib_user::Session;
use std::collections::HashMap;

pub struct DefaultLocationSignner<'a, T: LocationInfoGetterTrait> {
    location_info_getter: T,
    location_str: &'a Option<String>,
}

impl<'a, T: LocationInfoGetterTrait> DefaultLocationSignner<'a, T> {
    pub fn new(location_info_getter: T, location_str: &'a Option<String>) -> Self {
        Self {
            location_info_getter,
            location_str,
        }
    }
}
impl<T: LocationInfoGetterTrait> SignnerTrait<LocationSign> for DefaultLocationSignner<'_, T> {
    type ExtData<'e> = &'e Vec<Location>;

    fn sign<'b, Sessions: Iterator<Item = &'b Session> + Clone>(
        &mut self,
        sign: &LocationSign,
        sessions: Sessions,
    ) -> Result<HashMap<&'b Session, SignResult>, SignError> {
        let locations = self
            .location_info_getter
            .get_locations(sign, self.location_str);
        if locations.is_empty() {
            return Err(SignError::LocationError(
                "未获取到位置信息，请检查位置列表或检查输入。".to_owned(),
            ));
        }
        #[allow(clippy::mutable_key_type)]
        let mut map = HashMap::new();
        for session in sessions {
            let r = Self::sign_single(sign, session, &locations)?;
            map.insert(session, r);
        }
        Ok(map)
    }

    fn sign_single(
        sign: &LocationSign,
        session: &Session,
        locations: &Vec<Location>,
    ) -> Result<SignResult, SignError> {
        crate::signner::impls::utils::sign_single_retry(sign, session, (&(), locations))
    }
}
