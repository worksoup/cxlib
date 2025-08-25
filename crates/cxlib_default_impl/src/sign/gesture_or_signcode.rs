use crate::sign::RawSign;
use cxlib_protocol::{
    collect::SignProtocolTrait,
    utils::{SignHelperTrait, SignUrlHelper},
};
use cxlib_sign::{Analysis, AsRaw, SignError, SignResult, api2507::SignApi2507};
use cxlib_types::Session;
use serde::Serialize;

/// 手势签到。
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone, Serialize)]
pub struct GestureOrSigncodeSign {
    is_gesture: bool,
    raw_sign: RawSign,
}

impl GestureOrSigncodeSign {
    #[inline]
    pub fn new(is_gesture: bool, raw_sign: RawSign) -> Self {
        Self {
            is_gesture,
            raw_sign,
        }
    }
    #[inline]
    pub fn is_gesture(&self) -> bool {
        self.is_gesture
    }
    /// 检查签到码是否正确而不进行签到。
    ///
    /// 如果是手势签到，九宫格对应数字如下：
    /// ``` matlab
    /// 1 2 3
    /// 4 5 6
    /// 7 8 9
    /// ```
    #[inline]
    pub fn check_signcode<SignProtocol, U>(
        session: &Session,
        active_id: &str,
        signcode: &str,
    ) -> Result<Result<(), SignResult>, SignError>
    where
        SignProtocol: SignProtocolTrait,
    {
        let result_code = SignProtocol::check_signcode(session, active_id, signcode)?;
        if result_code == 1 {
            Ok(Ok(()))
        } else {
            Ok(Err(SignResult::Failure {
                msg: "签到码或手势不正确".into(),
                state_enum: None,
            }))
        }
    }
    pub fn into_raw(self) -> RawSign {
        self.raw_sign
    }
}

impl SignApi2507 for GestureOrSigncodeSign {
    type PreSignData = ();
    type Data = str;

    #[inline]
    fn sign_url<SignProtocol>(
        &self,
        session: &Session,
        _: &Self::PreSignData,
        data: &Self::Data,
    ) -> SignUrlHelper
    where
        SignProtocol: SignProtocolTrait,
    {
        SignProtocol::sign_in_url().signcode_sign_url(
            (session.uid(), session.fid(), session.name()),
            self.raw_sign.active_id(),
            data,
        )
    }
}
impl AsRaw for GestureOrSigncodeSign {
    #[inline]
    fn as_inner(&self) -> &RawSign {
        &self.raw_sign
    }
}
impl Analysis for GestureOrSigncodeSign {
    type AnalysisData = ();
}
