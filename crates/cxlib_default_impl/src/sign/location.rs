use crate::sign::{RawSign, SignTrait};
use cxlib_protocol::{
    collect::SignProtocolTrait,
    utils::{SignHelperTrait, SignUrlHelper},
};
use cxlib_sign::{AsRaw, need_pre_sign::NeedPreSign};
use cxlib_types::{
    Geoaddr, LocationPreprocessorTrait, Session, UnhandledGeoAddrWithRange,
    ext::UnhandledGeoAddrWithRangeExt,
};
use serde::Serialize;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone, Serialize)]
pub struct LocationSign {
    pub(crate) raw_sign: RawSign,
    pub(crate) preset_location: Option<UnhandledGeoAddrWithRange>,
}
impl LocationSign {
    /// 获取预设的位置，同时可以选择传入一个字符串，用来设置位置的名称。
    ///
    /// 注意该函数不会调用 [`set_location`](Self::set_location), 请手动调用。
    #[inline]
    pub fn get_preset_location(
        &self,
        preprocessor: &impl LocationPreprocessorTrait,
    ) -> Option<Geoaddr> {
        self.preset_location
            .as_ref()
            .map(|l| l.to_shifted_geoaddr(preprocessor))
    }
    #[inline]
    pub fn into_raw(self) -> RawSign {
        self.raw_sign
    }
}
impl SignTrait for LocationSign {
    type AnalysisSignData = ();
    type Data = Geoaddr;

    #[inline]
    fn sign_url<SignProtocol, U>(
        &self,
        session: &Session<U>,
        _: &(),
        data: &Geoaddr,
    ) -> SignUrlHelper
    where
        SignProtocol: SignProtocolTrait,
    {
        SignProtocol::ppt_sign_url().location_sign_url(
            (session.uid(), session.fid(), session.name()),
            (data.addr(), data.location().lat(), data.location().lon()),
            self.raw_sign.active_id(),
            self.preset_location.is_some(),
        )
    }
}
impl AsRaw for LocationSign {
    #[inline]
    fn as_inner(&self) -> &RawSign {
        &self.raw_sign
    }
}

impl NeedPreSign for LocationSign {}
