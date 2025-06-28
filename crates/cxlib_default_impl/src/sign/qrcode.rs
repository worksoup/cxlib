use crate::sign::{LocationSign, PreSignResult, RawSign, SignTrait};
use cxlib_protocol::{
    collect::{CaptchaProtocolTrait, SignProtocolTrait},
    utils::PPTSignHelper,
};
use cxlib_sign::SignError;
use cxlib_types::{Geoaddr, Session};
use derive_where::derive_where;
use log::info;
use serde::Serialize;

#[derive_where(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
#[derive(Serialize)]
pub struct QrCodeSign<SignProtocol> {
    pub(crate) is_refresh: bool,
    pub(crate) raw_sign: LocationSign<SignProtocol>,
    pub(crate) c: String,
}
impl<SignProtocol> QrCodeSign<SignProtocol> {
    pub fn as_location_sign_mut(&mut self) -> &mut LocationSign<SignProtocol> {
        &mut self.raw_sign
    }
    pub fn as_location_sign(&self) -> &LocationSign<SignProtocol> {
        &self.raw_sign
    }
    pub fn is_refresh(&self) -> bool {
        self.is_refresh
    }
}
impl<SignProtocol> SignTrait<SignProtocol> for QrCodeSign<SignProtocol> {
    type PreSignData = str;
    type Data = Option<Geoaddr>;

    fn sign_url<U>(
        &self,
        session: &Session<U>,
        enc: &str,
        location: &Option<Geoaddr>,
    ) -> PPTSignHelper
    where
        SignProtocol: SignProtocolTrait,
    {
        SignProtocol::qrcode_sign_url(
            (session.uid(), session.fid(), session.name()),
            enc,
            self.as_inner().active_id(),
            location.as_ref(),
        )
    }

    fn as_inner(&self) -> &RawSign<SignProtocol> {
        self.raw_sign.as_inner()
    }
    fn pre_sign<CaptchaProtocol, U>(
        &self,
        session: &Session<U>,
        enc: &str,
    ) -> Result<PreSignResult, SignError>
    where
        CaptchaProtocol: CaptchaProtocolTrait,
        SignProtocol: SignProtocolTrait,
    {
        let raw = self.as_inner();
        let active_id = raw.active_id();
        let uid = session.uid();
        let response_of_presign = SignProtocol::pre_sign_for_qrcode_sign(
            session,
            (raw.course().id(), raw.course().class_id()),
            active_id,
            uid,
            &self.c,
            enc,
        )?;
        info!("用户[{}]预签到已请求。", session.name());
        cxlib_sign::utils::analysis_after_presign::<CaptchaProtocol, SignProtocol, U>(
            active_id,
            session,
            response_of_presign,
        )
    }
}
