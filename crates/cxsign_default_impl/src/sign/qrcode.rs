use crate::sign::utils::sign_unchecked_with_location;
use crate::sign::{LocationSign, PreSignResult, RawSign, SignResult, SignTrait};
use cxsign_sign::utils::PPTSignHelper;
use cxsign_types::Location;
use cxsign_user::Session;
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct QrCodeSign {
    pub(crate) is_refresh: bool,
    pub(crate) raw_sign: LocationSign,
    pub(crate) enc: Option<String>,
    pub(crate) c: String,
}
impl QrCodeSign {
    pub fn set_enc(&mut self, enc: String) {
        self.enc = Some(enc)
    }
    pub fn set_location(&mut self, location: Location) {
        self.raw_sign.set_location(location)
    }
    pub fn as_location_sign_mut(&mut self) -> &mut LocationSign {
        &mut self.raw_sign
    }
    pub fn is_refresh(&self) -> bool {
        self.is_refresh
    }
}
impl SignTrait for QrCodeSign {
    type RuntimeData = Location;

    fn sign_url(&self, session: &Session, runtime_data: &Location) -> PPTSignHelper {
        let enc = unsafe { self.enc.as_ref().unwrap_unchecked() };
        cxsign_sign::protocol::qrcode_sign_url(
            session,
            enc,
            self.as_inner().active_id.as_str(),
            Some(runtime_data),
        )
    }

    fn as_inner(&self) -> &RawSign {
        self.raw_sign.as_inner()
    }
    fn is_ready_for_sign(&self) -> bool {
        self.enc.is_some()
    }
    fn pre_sign(&self, session: &Session) -> Result<PreSignResult, cxsign_error::Error> {
        let enc = self.enc.as_deref().unwrap_or("");
        let raw = self.as_inner();
        let active_id = raw.active_id.as_str();
        let uid = session.get_uid();
        let response_of_presign = cxsign_sign::protocol::pre_sign_for_qrcode_sign(
            session,
            raw.course.clone(),
            active_id,
            uid,
            &self.c,
            enc,
        )?;
        info!("用户[{}]预签到已请求。", session.get_stu_name());
        cxsign_sign::utils::analysis_after_presign(active_id, session, response_of_presign)
    }
    unsafe fn sign_unchecked(
        &self,
        session: &Session,
        pre_sign_result: PreSignResult,
    ) -> Result<SignResult, cxsign_error::Error> {
        match pre_sign_result {
            PreSignResult::Susses => Ok(SignResult::Susses),
            PreSignResult::Data(mut data) => sign_unchecked_with_location::<QrCodeSign>(
                self,
                &self.raw_sign.location,
                &self.raw_sign.preset_location,
                data.remove_first(),
                session,
            ),
        }
    }
}
