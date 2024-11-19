use crate::sign::{PreSignResult, RawSign, SignResult, SignTrait};
use cxlib_sign::utils::{try_secondary_verification, PPTSignHelper};
use cxlib_types::Photo;
use cxlib_user::Session;
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
    type RuntimeData = Photo;
    fn sign_url(&self, session: &Session, runtime_data: &Photo) -> PPTSignHelper {
        cxlib_sign::protocol::photo_sign_url(
            session,
            &self.as_inner().active_id,
            runtime_data.get_object_id(),
        )
    }

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
    ) -> Result<SignResult, cxlib_error::Error> {
        match pre_sign_result {
            PreSignResult::Susses => Ok(SignResult::Susses),
            PreSignResult::Data(mut data) => {
                let photo = self.photo.as_ref().unwrap();
                let url = self.sign_url(session, photo);
                try_secondary_verification::<Self>(session, url, &data.remove_first())
            }
        }
    }
}
