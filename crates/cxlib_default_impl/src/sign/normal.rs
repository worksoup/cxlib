use crate::sign::{RawSign, SignTrait};
use cxlib_protocol::{collect::SignProtocolTrait, utils::PPTSignHelper};
use cxlib_types::Session;
use derive_where::derive_where;
use serde::Serialize;

/// 普通签到。
#[derive_where(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
#[derive(Serialize)]
pub struct NormalSign<SignProtocol> {
    pub(crate) raw_sign: RawSign<SignProtocol>,
}

impl<SignProtocol> SignTrait<SignProtocol> for NormalSign<SignProtocol> {
    type PreSignData = ();
    type Data = ();

    fn sign_url<U>(&self, session: &Session<U>, _: &(), runtime_data: &Self::Data) -> PPTSignHelper
    where
        SignProtocol: SignProtocolTrait,
    {
        self.as_inner().sign_url(session, &(), runtime_data)
    }

    fn as_inner(&self) -> &RawSign<SignProtocol> {
        &self.raw_sign
    }
}
