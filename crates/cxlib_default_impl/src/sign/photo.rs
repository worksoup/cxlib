use crate::sign::{RawSign, SignTrait};
use cxlib_protocol::{collect::SignProtocolTrait, utils::PPTSignHelper};
use cxlib_types::{Photo, Session};
use derive_where::derive_where;
use serde::Serialize;
use std::marker::PhantomData;

#[derive_where(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
#[derive(Serialize)]
pub struct PhotoSign<SignProtocol, TypesProtocol> {
    raw_sign: RawSign<SignProtocol>,
    #[serde(skip)]
    _p: PhantomData<TypesProtocol>,
}
impl<SignProtocol, TypesProtocol> PhotoSign<SignProtocol, TypesProtocol> {
    pub fn new(raw_sign: RawSign<SignProtocol>) -> Self {
        Self {
            raw_sign,
            _p: PhantomData,
        }
    }
}
impl<SignProtocol, TypesProtocol> SignTrait<SignProtocol>
    for PhotoSign<SignProtocol, TypesProtocol>
{
    type PreSignData = ();
    type Data = Photo<TypesProtocol>;
    fn sign_url<U>(
        &self,
        session: &Session<U>,
        _: &(),
        runtime_data: &Photo<TypesProtocol>,
    ) -> PPTSignHelper
    where
        SignProtocol: SignProtocolTrait,
    {
        SignProtocol::photo_sign_url(
            (session.uid(), session.fid(), session.name()),
            self.as_inner().active_id(),
            runtime_data.get_object_id(),
        )
    }

    fn as_inner(&self) -> &RawSign<SignProtocol> {
        &self.raw_sign
    }
}
