use crate::sign::{RawSign, SignTrait};
use cxlib_sign::utils::PPTSignHelper;
use cxlib_types::{Location, LocationWithRange};
use cxlib_user::Session;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct LocationSign {
    pub(crate) raw_sign: RawSign,
    pub(crate) preset_location: Option<LocationWithRange>,
}
impl LocationSign {
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
    type PreSignData = ();
    type Data = Location;

    fn sign_url(&self, session: &Session, _: &(), data: &Location) -> PPTSignHelper {
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
}
