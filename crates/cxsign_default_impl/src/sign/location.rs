use crate::sign::utils::sign_unchecked_with_location;
use crate::sign::{PreSignResult, RawSign, SignResult, SignTrait};
use cxsign_types::{Location, LocationWithRange};
use cxsign_user::Session;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct LocationSign {
    pub(crate) raw_sign: RawSign,
    pub(crate) preset_location: Option<LocationWithRange>,
    pub(crate) location: Location,
}
impl LocationSign {
    /// 设置位置信息。
    pub fn set_location(&mut self, location: Location) {
        self.location = location
    }
    /// 获取预设的位置，同时可以选择传入一个字符串，用来设置位置的名称。
    ///
    /// 注意该函数不会调用 [`set_location`](Self::set_location), 请手动调用。
    pub fn get_preset_location(&self) -> Option<Location> {
        self.preset_location
            .as_ref()
            .map(|l| l.to_shifted_location())
    }
}
impl SignTrait for LocationSign {
    fn as_inner(&self) -> &RawSign {
        &self.raw_sign
    }
    unsafe fn sign_unchecked(
        &self,
        session: &Session,
        pre_sign_result: PreSignResult,
    ) -> Result<SignResult, cxsign_error::Error> {
        match pre_sign_result {
            PreSignResult::Susses => Ok(SignResult::Susses),
            PreSignResult::Data(captcha_id) => {
                let url_getter = |l: &Location| {
                    cxsign_sign::protocol::location_sign_url(
                        session,
                        l,
                        self.raw_sign.active_id.as_str(),
                        self.preset_location.is_some(),
                    )
                };
                sign_unchecked_with_location::<Self>(
                    url_getter,
                    &self.location,
                    &self.preset_location,
                    captcha_id.into_first(),
                    session,
                )
            }
        }
    }
    fn pre_sign_and_sign(&self, session: &Session) -> Result<SignResult, cxsign_error::Error> {
        let r = self.pre_sign(session)?;
        self.sign(session, r)
    }
}
