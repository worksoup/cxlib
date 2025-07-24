use crate::{sign::LocationSign, signner::LocationInfoGetterTrait};
use cxlib_captcha::CaptchaSolver;
use cxlib_protocol::collect::{CaptchaProtocolTrait, SignProtocolTrait};
use cxlib_sign::{SignError, SignResult, SignnerTrait};
use cxlib_types::{Geoaddr, LocationPreprocessorTrait, Session};
use std::collections::HashMap;

pub struct DefaultLocationSignner<'a, T: LocationInfoGetterTrait, PP: LocationPreprocessorTrait> {
    location_info_getter: T,
    location_str: &'a Option<String>,
    preprocessor: &'a PP,
}

impl<'a, T: LocationInfoGetterTrait, PP: LocationPreprocessorTrait>
    DefaultLocationSignner<'a, T, PP>
{
    pub fn new(
        location_info_getter: T,
        location_str: &'a Option<String>,
        preprocessor: &'a PP,
    ) -> Self {
        Self {
            location_info_getter,
            location_str,
            preprocessor,
        }
    }
}
impl<CaptchaProtocol, SignProtocol, T: LocationInfoGetterTrait, PP: LocationPreprocessorTrait>
    SignnerTrait<LocationSign, CaptchaProtocol, SignProtocol> for DefaultLocationSignner<'_, T, PP>
where
    CaptchaProtocol: CaptchaProtocolTrait,
    SignProtocol: SignProtocolTrait,
{
    type ExtData<'e> = &'e Vec<Geoaddr>;

    fn sign<'b, U, Sessions: Iterator<Item = &'b Session<U>> + Clone>(
        &mut self,
        sign: &LocationSign,
        sessions: Sessions,
        captcha_solver: &CaptchaSolver,
    ) -> Result<HashMap<&'b Session<U>, SignResult>, SignError> {
        let locations = self.location_info_getter.get_locations(
            sign,
            self.location_str,
            self.preprocessor,
        );
        if locations.is_empty() {
            return Err(SignError::LocationError(
                "未获取到位置信息，请检查位置列表或检查输入。".to_owned(),
            ));
        }
        #[allow(clippy::mutable_key_type)]
        let mut map = HashMap::new();
        for session in sessions {
            let r =
                <Self as SignnerTrait<LocationSign, CaptchaProtocol, SignProtocol>>::sign_single(
                    sign,
                    session,
                    captcha_solver,
                    &locations,
                )?;
            map.insert(session, r);
        }
        Ok(map)
    }

    fn sign_single<U>(
        sign: &LocationSign,
        session: &Session<U>,
        captcha_solver: &CaptchaSolver,
        locations: &Vec<Geoaddr>,
    ) -> Result<SignResult, SignError> {
        crate::signner::impls::utils::sign_single_retry::<
            CaptchaProtocol,
            SignProtocol,
            _,
            _,
            _,
            _,
            _,
        >(sign, session, (&(), locations), captcha_solver)
    }
}
