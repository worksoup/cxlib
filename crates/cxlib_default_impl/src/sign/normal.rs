use crate::sign::{RawSign, SignTrait};
use cxlib_sign::utils::PPTSignHelper;
use cxlib_user::Session;
use serde::{Deserialize, Serialize};

/// 普通签到。
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct NormalSign {
    pub(crate) raw_sign: RawSign,
}

impl SignTrait for NormalSign {
    type RuntimeData = ();

    fn sign_url(&self, session: &Session, runtime_data: &Self::RuntimeData) -> PPTSignHelper {
        self.as_inner().sign_url(session, runtime_data)
    }

    fn as_inner(&self) -> &RawSign {
        &self.raw_sign
    }
}
