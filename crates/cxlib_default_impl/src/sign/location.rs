use crate::sign::utils::sign_unchecked_with_location;
use crate::sign::{PreSignResult, RawSign, SignResult, SignTrait};
use cxlib_sign::utils::PPTSignHelper;
use cxlib_types::{Location, LocationWithRange};
use cxlib_user::Session;
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
    type RuntimeData = Location;

    fn sign_url(&self, session: &Session, data: &Location) -> PPTSignHelper {
        cxlib_sign::protocol::location_sign_url(
            session,
            data,
            self.raw_sign.active_id.as_str(),
            self.preset_location.is_some(),
        )
    }

    fn as_inner(&self) -> &RawSign {
        &self.raw_sign
    }
    unsafe fn sign_unchecked(
        &self,
        session: &Session,
        pre_sign_result: PreSignResult,
    ) -> Result<SignResult, cxlib_error::Error> {
        match pre_sign_result {
            PreSignResult::Susses => Ok(SignResult::Susses),
            PreSignResult::Data(mut data) => sign_unchecked_with_location::<Self>(
                self,
                &self.location,
                &self.preset_location,
                data.take_first(),
                session,
            ),
        }
    }
    fn pre_sign_and_sign(&self, session: &Session) -> Result<SignResult, cxlib_error::Error> {
        let r = self.pre_sign(session)?;
        self.sign(session, r)
    }
}
