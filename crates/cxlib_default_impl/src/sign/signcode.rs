use crate::sign::RawSign;
use cxlib_sign::GestureOrSigncodeSignTrait;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct SigncodeSign {
    pub(crate) raw_sign: RawSign,
}

impl GestureOrSigncodeSignTrait for SigncodeSign {
    fn as_inner(&self) -> &RawSign {
        &self.raw_sign
    }
}
