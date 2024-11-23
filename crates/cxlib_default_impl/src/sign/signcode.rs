use crate::sign::{GestureOrSigncodeSignTrait, PreSignResult, RawSign, SignResult, SignTrait};
use cxlib_sign::protocol::signcode_sign_url;
use cxlib_sign::utils::PPTSignHelper;
use cxlib_user::Session;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct SigncodeSign {
    pub(crate) raw_sign: RawSign,
    pub(crate) signcode: Option<String>,
}
impl SigncodeSign {
    pub fn check(&self, session: &Session) -> bool {
        self.signcode.as_ref().map_or(false, |signcode| {
            Self::check_signcode(session, &self.raw_sign.active_id, signcode).unwrap_or(false)
        })
    }
    pub fn set_signcode(&mut self, signcode: String) {
        self.signcode = Some(signcode)
    }
}
impl SignTrait for SigncodeSign {
    type RuntimeData = String;
    fn sign_url(&self, session: &Session, runtime_data: &Self::RuntimeData) -> PPTSignHelper {
        signcode_sign_url(session, &self.as_inner().active_id, runtime_data)
    }

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
    ) -> Result<SignResult, cxlib_error::Error> {
        match pre_sign_result {
            PreSignResult::Susses => Ok(SignResult::Susses),
            PreSignResult::Data(mut data) => Ok(self.sign_with_signcode(
                session,
                unsafe { self.signcode.as_ref().unwrap_unchecked() },
                data.take_first(),
            )?),
        }
    }
}
