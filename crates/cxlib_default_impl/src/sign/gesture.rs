use crate::sign::RawSign;
use cxlib_sign::GestureOrSigncodeSignTrait;
use serde::{Deserialize, Serialize};

/// 手势签到。
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct GestureSign {
    pub(crate) raw_sign: RawSign,
}

impl GestureOrSigncodeSignTrait for GestureSign {
    fn as_inner(&self) -> &RawSign {
        &self.raw_sign
    }
}
