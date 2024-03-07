use crate::activity::sign::base::BaseSign;
use crate::activity::sign::{Enum签到后状态, Enum签到结果, SignTrait};
use crate::user::session::Session;
use ureq::Error;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NormalSign {
    pub(crate) base_sign: BaseSign,
}

impl SignTrait for NormalSign {
    fn is_valid(&self) -> bool {
        self.base_sign.is_valid()
    }

    fn get_attend_info(&self, session: &Session) -> Result<Enum签到后状态, Error> {
        self.base_sign.get_attend_info(session)
    }

    fn pre_sign(&self, session: &Session) -> Result<Enum签到结果, Error> {
        self.base_sign.pre_sign(session)
    }

    fn sign(&self, session: &Session) -> Result<Enum签到结果, Error> {
        self.base_sign.sign(session)
    }
}
