use crate::sign::{LocationSign, RawSign, SignTrait};
use cxlib_protocol::{
    collect::{CaptchaProtocolTrait, PreSignResult, SignProtocolTrait},
    utils::{SignHelperTrait, SignUrlHelper},
};
use cxlib_sign::SignError;
use cxlib_types::{Geoaddr, Session};
use log::info;
use serde::Serialize;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone, Serialize)]
pub struct QrCodeSign {
    pub(crate) is_refresh: bool,
    pub(crate) raw_sign: LocationSign,
    pub(crate) c: String,
}
impl QrCodeSign {
    #[inline]
    pub fn as_location_sign_mut(&mut self) -> &mut LocationSign {
        &mut self.raw_sign
    }
    #[inline]
    pub fn as_location_sign(&self) -> &LocationSign {
        &self.raw_sign
    }
    #[inline]
    pub fn is_refresh(&self) -> bool {
        self.is_refresh
    }
    #[inline]
    pub fn into_raw(self) -> RawSign {
        self.raw_sign.into_raw()
    }
}
impl SignTrait for QrCodeSign {
    type PreSignData = str;
    type Data = Option<Geoaddr>;

    #[inline]
    fn sign_url<SignProtocol, U>(
        &self,
        session: &Session<U>,
        enc: &str,
        location: &Option<Geoaddr>,
    ) -> SignUrlHelper
    where
        SignProtocol: SignProtocolTrait,
    {
        SignProtocol::ppt_sign_url().qrcode_sign_url(
            (session.uid(), session.fid(), session.name()),
            enc,
            self.as_inner().active_id(),
            location.as_ref(),
        )
    }

    #[inline]
    fn as_inner(&self) -> &RawSign {
        self.raw_sign.as_inner()
    }
    fn pre_sign<CaptchaProtocol, SignProtocol, U>(
        &self,
        session: &Session<U>,
        enc: &str,
    ) -> Result<PreSignResult, SignError>
    where
        CaptchaProtocol: CaptchaProtocolTrait,
        SignProtocol: SignProtocolTrait,
    {
        let raw = SignTrait::as_inner(self);
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
        Ok(response_of_presign.analysis::<CaptchaProtocol, SignProtocol>(session, active_id)?)
    }
}
