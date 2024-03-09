use crate::activity::sign::base::BaseSign;
use crate::activity::sign::{SignState, SignResult, SignTrait};
use crate::photo::Photo;
use crate::protocol;
use crate::user::session::Session;
use ureq::Error;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhotoSign {
    pub(crate) base_sign: BaseSign,
    pub(crate) photo: Option<Photo>,
}

impl SignTrait for PhotoSign {
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
        if let Some(photo) = self.photo.as_ref() {
            let r = protocol::photo_sign(
                session,
                session.get_uid(),
                session.get_fid(),
                session.get_stu_name(),
                self.base_sign.active_id.as_str(),
                photo.get_object_id(),
            )?;
            Ok(Self::通过文本判断签到结果(
                &r.into_string().unwrap(),
            ))
        } else {
            Ok(SignResult::Fail {
                msg: "".to_string(),
            })
        }
    }
}