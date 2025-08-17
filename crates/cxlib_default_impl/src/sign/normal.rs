use crate::sign::{RawSign, SignTrait};
use cxlib_protocol::{collect::SignProtocolTrait, utils::SignUrlHelper};
use cxlib_sign::api2507::SignApi2507;
use cxlib_types::Session;
use serde::Serialize;

/// 普通签到。
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone, Serialize)]
pub struct NormalSign {
    pub(crate) raw_sign: RawSign,
}
impl NormalSign {
    #[inline]
    pub fn into_raw(self) -> RawSign {
        self.raw_sign
    }
}
impl SignApi2507 for NormalSign {
    type PreSignData = ();
    type Data = ();

    #[inline]
    fn sign_url<SignProtocol, U>(
        &self,
        session: &Session<U>,
        _: &(),
        runtime_data: &Self::Data,
    ) -> SignUrlHelper
    where
        SignProtocol: SignProtocolTrait,
    {
        self.raw_sign
            .sign_url::<SignProtocol, _>(session, &(), runtime_data)
    }

    #[inline]
    fn as_inner(&self) -> &RawSign {
        &self.raw_sign
    }
}
