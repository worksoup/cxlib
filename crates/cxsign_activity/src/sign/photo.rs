use crate::protocol;
use crate::sign::{PreSignResult, RawSign, SignResult, SignTrait};
use cxsign_types::Photo;
use cxsign_user::Session;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Serialize, Deserialize)]
pub struct PhotoSign {
    pub(crate) raw_sign: RawSign,
    pub(crate) photo: Option<Photo>,
}
impl PhotoSign {
    pub fn set_photo(&mut self, photo: Photo) {
        self.photo = Some(photo)
    }
}
impl SignTrait for PhotoSign {
    fn as_inner(&self) -> &RawSign {
        &self.raw_sign
    }

    fn is_ready_for_sign(&self) -> bool {
        self.photo.is_some()
    }
    unsafe fn sign_unchecked(
        &self,
        session: &Session,
        pre_sign_result: PreSignResult,
    ) -> Result<SignResult, cxsign_error::Error> {
        match pre_sign_result {
            PreSignResult::Susses => Ok(SignResult::Susses),
            _ => {
                let photo = self.photo.as_ref().unwrap();
                let r = protocol::photo_sign(
                    session,
                    self.raw_sign.active_id.as_str(),
                    photo.get_object_id(),
                )?;
                Ok(self.guess_sign_result_by_text(&r.into_string().unwrap()))
            }
        }
    }
}
