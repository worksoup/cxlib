use crate::sign::{RawSign, SignTrait};
use cxlib_protocol::{collect::SignProtocolTrait, utils::PPTSignHelper};
use cxlib_types::Session;
use serde::Serialize;

/// 普通签到。
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone, Serialize)]
pub struct NormalSign {
    pub(crate) raw_sign: RawSign,
}

impl SignTrait for NormalSign {
    type PreSignData = ();
    type Data = ();

    fn sign_url<SignProtocol, U>(
        &self,
        session: &Session<U>,
        _: &(),
        runtime_data: &Self::Data,
    ) -> PPTSignHelper
    where
        SignProtocol: SignProtocolTrait,
    {
        self.raw_sign
            .sign_url::<SignProtocol, _>(session, &(), runtime_data)
    }

    fn as_inner(&self) -> &RawSign {
        &self.raw_sign
    }
}
