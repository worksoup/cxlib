use crate::sign::{RawSign, SignTrait};
use cxlib_protocol::{collect::SignProtocolTrait, utils::PPTSignHelper};
use cxlib_types::{Geoaddr, LocationPreprocessorTrait, LocationWithRange, Session};
use derive_where::derive_where;
use serde::Serialize;

#[derive_where(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
#[derive(Serialize)]
pub struct LocationSign<SignProtocol> {
    pub(crate) raw_sign: RawSign<SignProtocol>,
    pub(crate) preset_location: Option<LocationWithRange>,
}
impl<SignProtocol> LocationSign<SignProtocol> {
    /// 获取预设的位置，同时可以选择传入一个字符串，用来设置位置的名称。
    ///
    /// 注意该函数不会调用 [`set_location`](Self::set_location), 请手动调用。
    pub fn get_preset_location(
        &self,
        preprocessor: &impl LocationPreprocessorTrait,
    ) -> Option<Geoaddr> {
        self.preset_location
            .as_ref()
            .map(|l| l.to_shifted_geoaddr(preprocessor))
    }
}
impl<SignProtocol> SignTrait<SignProtocol> for LocationSign<SignProtocol> {
    type PreSignData = ();
    type Data = Geoaddr;

    fn sign_url<U>(&self, session: &Session<U>, _: &(), data: &Geoaddr) -> PPTSignHelper
    where
        SignProtocol: SignProtocolTrait,
    {
        SignProtocol::location_sign_url(
            (session.uid(), session.fid(), session.name()),
            (data.addr(), data.location().lat(), data.location().lon()),
            self.raw_sign.active_id(),
            self.preset_location.is_some(),
        )
    }

    fn as_inner(&self) -> &RawSign<SignProtocol> {
        &self.raw_sign
    }
}
