use crate::sign::RawSign;
use cxlib_protocol::{
    collect::SignProtocolTrait,
    utils::{SignHelperTrait, SignUrlHelper},
};
use cxlib_sign::{Analysis, AsRaw, SignTrait, api2507::SignApi2507};
use cxlib_types::{Photo, Session};
use derive_where::derive_where;
use serde::Serialize;
use std::marker::PhantomData;

#[derive_where(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
#[derive(Serialize)]
pub struct PhotoSign<NetdiskProtocol> {
    raw_sign: RawSign,
    #[serde(skip)]
    _p: PhantomData<NetdiskProtocol>,
}
impl<NetdiskProtocol> PhotoSign<NetdiskProtocol> {
    #[inline]
    pub fn new(raw_sign: RawSign) -> Self {
        Self {
            raw_sign,
            _p: PhantomData,
        }
    }

    #[inline]
    pub fn into_raw(self) -> RawSign {
        self.raw_sign
    }
}
impl<NetdiskProtocol> SignApi2507 for PhotoSign<NetdiskProtocol> {
    type PreSignData = ();
    type Data = Photo<NetdiskProtocol>;
    #[inline]
    fn sign_url<SignProtocol, U>(
        &self,
        session: &Session<U>,
        _: &(),
        runtime_data: &Photo<NetdiskProtocol>,
    ) -> SignUrlHelper
    where
        SignProtocol: SignProtocolTrait,
    {
        SignProtocol::sign_in_url().photo_sign_url(
            (session.uid(), session.fid(), session.name()),
            self.raw_sign.active_id(),
            runtime_data.get_object_id(),
        )
    }
}
impl<N> AsRaw for PhotoSign<N> {
    #[inline]
    fn as_inner(&self) -> &RawSign {
        &self.raw_sign
    }
}
impl<N> Analysis for PhotoSign<N> {
    type AnalysisData = <Self as SignTrait>::AnalysisSignData;
}
