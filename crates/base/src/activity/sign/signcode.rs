use crate::activity::sign::base::BaseSign;
use crate::activity::sign::{SignResult, SignState, SignTrait};
use crate::user::session::Session;
use ureq::Error;
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SigncodeSign {
    pub(crate) base_sign: BaseSign,
}

impl SignTrait for SigncodeSign {
    fn is_valid(&self) -> bool {
        self.base_sign.is_valid()
    }

    fn get_attend_info(&self, session: &Session) -> Result<SignState, Error> {
        self.base_sign.get_attend_info(session)
    }

    unsafe fn sign_internal(&self, session: &Session) -> Result<SignResult, Error> {
        unsafe { self.base_sign.sign_internal(session) }
    }

    fn sign(&self, session: &Session) -> Result<SignResult, Error> {
        self.base_sign.sign(session)
    }
}
