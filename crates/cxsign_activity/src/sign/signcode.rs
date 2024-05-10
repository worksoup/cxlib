use crate::sign::{PreSignResult, RawSign, SignResult, SignTrait};
use cxsign_user::Session;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct SigncodeSign {
    pub(crate) raw_sign: RawSign,
    pub(crate) signcode: Option<String>,
}
impl SigncodeSign {
    pub fn check(&self, session: &Session) -> bool {
        self.signcode.as_ref().map_or(false, |signcode| {
            RawSign::check_signcode(session, &self.raw_sign.active_id, signcode).unwrap_or(false)
        })
    }
    pub fn set_signcode(&mut self, signcode: String) {
        self.signcode = Some(signcode)
    }
}
impl SignTrait for SigncodeSign {
    fn as_inner(&self) -> &RawSign {
        &self.raw_sign
    }
    fn is_ready_for_sign(&self) -> bool {
        self.signcode.is_some()
    }

    unsafe fn sign_unchecked(
        &self,
        session: &Session,
        pre_sign_result: PreSignResult,
    ) -> Result<SignResult, cxsign_error::Error> {
        match pre_sign_result {
            PreSignResult::Susses => Ok(SignResult::Susses),
            _ => Ok(self.as_inner().sign_with_signcode(session, unsafe {
                self.signcode.as_ref().unwrap_unchecked()
            })?),
        }
    }
}
