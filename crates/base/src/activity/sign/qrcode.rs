use crate::activity::sign::base::BaseSign;
use crate::activity::sign::{SignResult, SignState, SignTrait};
use crate::location::Location;
use crate::protocol;
use crate::user::session::Session;
use ureq::Error;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RefreshQrCodeSign {
    pub(crate) base_sign: BaseSign,
    pub(crate) enc: Option<String>,
    pub(crate) c: Option<String>,
    pub(crate) location: Option<Location>,
}
impl SignTrait for RefreshQrCodeSign {
    fn is_ready_for_sign(&self) -> bool {
        self.c.is_some() && self.enc.is_some()
    }
    fn is_valid(&self) -> bool {
        self.base_sign.is_valid()
    }

    fn get_attend_info(&self, session: &Session) -> Result<SignState, Error> {
        self.base_sign.get_attend_info(session)
    }

    unsafe fn sign_internal(&self, session: &Session) -> Result<SignResult, Error> {
        let c = unsafe { self.c.as_ref().unwrap_unchecked() };
        let enc = unsafe { self.enc.as_ref().unwrap_unchecked() };
        let r = self
            .base_sign
            .presign_for_refresh_qrcode_sign(c, enc, session);
        if let Ok(a) = r.as_ref()
            && !a.is_susses()
        {
            let r = protocol::qrcode_sign(
                session,
                enc,
                session.get_uid(),
                session.get_fid(),
                session.get_stu_name(),
                self.base_sign.active_id.as_str(),
                &self.location,
            )?;
            Ok(Self::通过文本判断签到结果(
                &r.into_string().unwrap(),
            ))
        } else {
            r
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NormalQrCodeSign {
    pub(crate) base_sign: BaseSign,
}
impl SignTrait for NormalQrCodeSign {
    fn is_valid(&self) -> bool {
        self.base_sign.is_valid()
    }

    fn get_attend_info(&self, session: &Session) -> Result<SignState, Error> {
        self.base_sign.get_attend_info(session)
    }

    unsafe fn sign_internal(&self, session: &Session) -> Result<SignResult, Error> {
        unsafe { self.base_sign.sign_internal(session) }
    }

    fn sign(&self, session: &Session) -> Result<SignResult, Error> {
        self.base_sign.sign(session)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum QrCodeSign {
    RefreshQrCodeSign(RefreshQrCodeSign),
    NormalQrCodeSign(NormalQrCodeSign),
}
impl SignTrait for QrCodeSign {
    fn is_valid(&self) -> bool {
        match self {
            QrCodeSign::RefreshQrCodeSign(a) => a.is_valid(),
            QrCodeSign::NormalQrCodeSign(a) => a.is_valid(),
        }
    }

    fn get_attend_info(&self, session: &Session) -> Result<SignState, Error> {
        match self {
            QrCodeSign::RefreshQrCodeSign(a) => a.get_attend_info(session),
            QrCodeSign::NormalQrCodeSign(a) => a.get_attend_info(session),
        }
    }

    unsafe fn sign_internal(&self, session: &Session) -> Result<SignResult, Error> {
        unsafe {
            match self {
                QrCodeSign::RefreshQrCodeSign(a) => a.sign_internal(session),
                QrCodeSign::NormalQrCodeSign(a) => a.sign_internal(session),
            }
        }
    }

    fn sign(&self, session: &Session) -> Result<SignResult, Error> {
        match self {
            QrCodeSign::RefreshQrCodeSign(a) => a.sign(session),
            QrCodeSign::NormalQrCodeSign(a) => a.sign(session),
        }
    }
}
