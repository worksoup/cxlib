use crate::sign::{LocationSign, PreSignResult, RawSign, SignTrait};
use cxlib_error::SignError;
use cxlib_protocol::collect::sign as protocol;
use cxlib_protocol::utils::PPTSignHelper;
use cxlib_types::Location;
use cxlib_user::Session;
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct QrCodeSign {
    pub(crate) is_refresh: bool,
    pub(crate) raw_sign: LocationSign,
    pub(crate) c: String,
}
impl QrCodeSign {
    pub fn as_location_sign_mut(&mut self) -> &mut LocationSign {
        &mut self.raw_sign
    }
    pub fn as_location_sign(&self) -> &LocationSign {
        &self.raw_sign
    }
    pub fn is_refresh(&self) -> bool {
        self.is_refresh
    }
}
impl SignTrait for QrCodeSign {
    type PreSignData = str;
    type Data = Option<Location>;

    fn sign_url(&self, session: &Session, enc: &str, location: &Option<Location>) -> PPTSignHelper {
        protocol::qrcode_sign_url(
            (session.get_uid(), session.get_fid(), session.get_stu_name()),
            enc,
            self.as_inner().active_id.as_str(),
            location
                .as_ref()
                .map(|l| (l.get_addr(), l.get_lat(), l.get_lon(), l.get_alt())),
        )
    }

    fn as_inner(&self) -> &RawSign {
        self.raw_sign.as_inner()
    }
    fn pre_sign(&self, session: &Session, enc: &str) -> Result<PreSignResult, SignError> {
        let raw = self.as_inner();
        let active_id = raw.active_id.as_str();
        let uid = session.get_uid();
        let response_of_presign = protocol::pre_sign_for_qrcode_sign(
            session,
            (raw.course.get_id(), raw.course.get_class_id()),
            active_id,
            uid,
            &self.c,
            enc,
        )?;
        info!("用户[{}]预签到已请求。", session.get_stu_name());
        cxlib_sign::utils::analysis_after_presign(active_id, session, response_of_presign)
    }
}
