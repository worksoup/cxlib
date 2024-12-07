use crate::{sign::LocationSign, signner::LocationInfoGetterTrait};
use cxlib_error::Error;
use cxlib_sign::{SignResult, SignTrait, SignnerTrait};
use cxlib_types::Location;
use cxlib_user::Session;
use log::error;
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
    type ExtData<'e> = &'e Location;

    fn sign<'b, Sessions: Iterator<Item = &'b Session> + Clone>(
        &mut self,
        sign: &mut LocationSign,
        sessions: Sessions,
    ) -> Result<HashMap<&'b Session, SignResult>, Error> {
        let location = self
            .location_info_getter
            .get_locations(sign, self.location_str)
            .ok_or_else(|| {
                error!("未获取到位置信息，请检查位置列表或检查输入。");
                Error::LocationError
            })?;
        #[allow(clippy::mutable_key_type)]
        let mut map = HashMap::new();
        for session in sessions {
            let r = Self::sign_single(sign, session, &location)?;
            map.insert(session, r);
        }
        Ok(map)
    }

    fn sign_single(
        sign: &mut LocationSign,
        session: &Session,
        location: &Location,
    ) -> Result<SignResult, Error> {
        sign.pre_sign_and_sign(session, &(), location)
    }
}
