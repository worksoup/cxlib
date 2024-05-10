use crate::sign::{PreSignResult, RawSign, SignResult, SignTrait};
use cxsign_user::Session;
use serde::{Deserialize, Serialize};
/// 手势签到。
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct GestureSign {
    pub(crate) raw_sign: RawSign,
    pub(crate) gesture: Option<String>,
}
impl GestureSign {
    /// 检查签到码是否正确而不进行签到。
    pub fn check(&self, session: &Session) -> bool {
        self.gesture.as_ref().map_or(false, |signcode| {
            RawSign::check_signcode(session, &self.raw_sign.active_id, signcode).unwrap_or(false)
        })
    }
    /// 设置手势。
    ///
    /// 九宫格对应数字如下：
    /// ``` matlab
    /// 1 2 3
    /// 4 5 6
    /// 7 8 9
    /// ```
    pub fn set_gesture(&mut self, gesture: String) {
        self.gesture = Some(gesture)
    }
}
impl SignTrait for GestureSign {
    fn as_inner(&self) -> &RawSign {
        &self.raw_sign
    }
    fn is_ready_for_sign(&self) -> bool {
        self.gesture.is_some()
    }
    unsafe fn sign_unchecked(
        &self,
        session: &Session,
        pre_sign_result: PreSignResult,
    ) -> Result<SignResult, cxsign_error::Error> {
        match pre_sign_result {
            PreSignResult::Susses => Ok(SignResult::Susses),
            _ => self
                .as_inner()
                .sign_with_signcode(session, unsafe { self.gesture.as_ref().unwrap_unchecked() }),
        }
    }
}
