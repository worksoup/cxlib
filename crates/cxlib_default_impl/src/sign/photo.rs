use crate::sign::{PreSignResult, RawSign, SignResult, SignTrait};
use cxlib_sign::utils::{try_secondary_verification, PPTSignHelper};
use cxlib_types::Photo;
use cxlib_user::Session;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Serialize, Deserialize)]
pub struct PhotoSign {
    pub(crate) raw_sign: RawSign,
}
impl PhotoSign {}
impl SignTrait for PhotoSign {
    type PreSignData = ();
    type Data = Photo;
    fn sign_url(&self, session: &Session,_: &(), runtime_data: &Photo) -> PPTSignHelper {
        cxlib_sign::protocol::photo_sign_url(
            session,
            &self.as_inner().active_id,
            runtime_data.get_object_id(),
        )
    }

    fn as_inner(&self) -> &RawSign {
        &self.raw_sign
    }
}
