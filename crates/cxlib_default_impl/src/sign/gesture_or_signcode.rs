use crate::sign::RawSign;
use cxlib_error_utils::CxlibResultUtils;
use cxlib_protocol::{collect::SignProtocolTrait, utils::PPTSignHelper};
use cxlib_sign::{SignError, SignResult, SignTrait};
use cxlib_types::Session;
use derive_where::derive_where;
use serde::{Deserialize, Serialize};

/// 手势签到。
#[derive_where(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
#[derive(Serialize)]
pub struct GestureOrSigncodeSign<SignProtocol> {
    is_gesture: bool,
    raw_sign: RawSign<SignProtocol>,
}

impl<SignProtocol> GestureOrSigncodeSign<SignProtocol> {
    pub fn new(is_gesture: bool, raw_sign: RawSign<SignProtocol>) -> Self {
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
    pub fn check_signcode<U>(
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
}

impl<SignProtocol> SignTrait<SignProtocol> for GestureOrSigncodeSign<SignProtocol> {
    type PreSignData = ();
    type Data = str;

    fn sign_url<U>(
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
            self.as_inner().active_id(),
            data,
        )
    }

    fn as_inner(&self) -> &RawSign<SignProtocol> {
        &self.raw_sign
    }
}
