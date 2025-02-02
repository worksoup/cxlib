use crate::sign::{RawSign, SignTrait};
use cxlib_protocol::{collect::sign as protocol, utils::PPTSignHelper};
use cxlib_types::{Photo, Session};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Serialize, Deserialize)]
pub struct PhotoSign {
    pub(crate) raw_sign: RawSign,
}
impl PhotoSign {}
impl SignTrait for PhotoSign {
    type PreSignData = ();
    type Data = Photo;
    fn sign_url(&self, session: &Session, _: &(), runtime_data: &Photo) -> PPTSignHelper {
        protocol::photo_sign_url(
            (session.uid(), session.fid(), session.name()),
            &self.as_inner().active_id,
            runtime_data.get_object_id(),
        )
    }

    fn as_inner(&self) -> &RawSign {
        &self.raw_sign
    }
}
