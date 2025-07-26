use crate::sign::RawSign;
use cxlib_error_utils::CxlibResultUtils;
use cxlib_protocol::{collect::SignProtocolTrait, utils::PPTSignHelper};
use cxlib_sign::{SignError, SignResult, SignTrait};
use cxlib_types::Session;
use serde::{Deserialize, Serialize};

/// 手势签到。
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone, Serialize)]
pub struct GestureOrSigncodeSign {
    is_gesture: bool,
    raw_sign: RawSign,
}

impl GestureOrSigncodeSign {
    pub fn new(is_gesture: bool, raw_sign: RawSign) -> Self {
        Self {
            is_gesture,
            raw_sign,
        }
    }
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
    pub fn check_signcode<SignProtocol, U>(
        session: &Session<U>,
        active_id: &str,
        signcode: &str,
    ) -> Result<Result<(), SignResult>, SignError>
    where
        SignProtocol: SignProtocolTrait,
    {
        #[derive(Deserialize)]
        struct CheckR {
            #[allow(unused)]
            result: i64,
        }
        let CheckR { result } = SignProtocol::check_signcode(session, active_id, signcode)?
            .into_body()
            .read_json()
            .log_unwrap();
        if result == 1 {
            Ok(Ok(()))
        } else {
            Ok(Err(SignResult::Fail {
                msg: "签到码或手势不正确".into(),
            }))
        }
    }
    pub fn into_raw(self) -> RawSign {
        self.raw_sign
    }
}

impl SignTrait for GestureOrSigncodeSign {
    type PreSignData = ();
    type Data = str;

    fn sign_url<SignProtocol, U>(
        &self,
        session: &Session<U>,
        _: &Self::PreSignData,
        data: &Self::Data,
    ) -> PPTSignHelper
    where
        SignProtocol: SignProtocolTrait,
    {
        SignProtocol::signcode_sign_url(
            (session.uid(), session.fid(), session.name()),
            self.raw_sign.active_id(),
            data,
        )
    }

    fn as_inner(&self) -> &RawSign {
        &self.raw_sign
    }
}
