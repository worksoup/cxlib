use crate::activity::sign::base::BaseSign;
use crate::activity::sign::{SignState, SignResult, SignTrait};
use crate::user::session::Session;
use ureq::Error;
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LocationSign {
    pub(crate) base_sign: BaseSign,
}

impl SignTrait for LocationSign {
    fn is_valid(&self) -> bool {
        self.base_sign.is_valid()
    }

    fn get_attend_info(&self, session: &Session) -> Result<SignState, Error> {
        self.base_sign.get_attend_info(session)
    }

    fn pre_sign(&self, session: &Session) -> Result<SignResult, Error> {
        self.base_sign.pre_sign(session)
    }

    fn sign(&self, session: &Session) -> Result<SignResult, Error> {
        self.base_sign.sign(session)
    }
}
