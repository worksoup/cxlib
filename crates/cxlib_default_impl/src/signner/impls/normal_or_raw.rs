use crate::sign::NormalSign;
use cxlib_captcha::CaptchaSolverTrait;
use cxlib_protocol::collect::{CaptchaProtocolTrait, SignProtocolTrait};
use cxlib_sign::{AsRaw, SignError, SignResult, SignTrait, SignnerTrait};
use cxlib_types::{RawSign, Session, SessionUserInfo};
use std::collections::HashMap;

pub struct DefaultNormalOrRawSignner;

#[inline]
fn sign_single_<CaptchaSolver: CaptchaSolverTrait, CaptchaProtocol, SignProtocol>(
    sign: &RawSign,
    session: &Session,
) -> Result<SignResult, SignError>
where
    CaptchaProtocol: CaptchaProtocolTrait,
    SignProtocol: SignProtocolTrait,
{
    sign.check_state_and_do_sign::<CaptchaSolver, CaptchaProtocol, SignProtocol>(session, &(), &())
}
#[inline]
fn sign_<
    'a,
    CaptchaSolver: CaptchaSolverTrait,
    CaptchaProtocol,
    SignProtocol,
    Sessions: Iterator<Item = &'a Session>,
>(
    sign: &RawSign,
    sessions: Sessions,
) -> Result<HashMap<&'a SessionUserInfo, SignResult>, SignError>
where
    CaptchaProtocol: CaptchaProtocolTrait,
    SignProtocol: SignProtocolTrait,
{
    let mut map = HashMap::new();
    for session in sessions {
        let a = sign_single_::<CaptchaSolver, CaptchaProtocol, SignProtocol>(sign, session)?;
        map.insert(session.user_info(), a);
    }
    Ok(map)
}

impl<CaptchaSolver, CaptchaProtocol, SignProtocol>
    SignnerTrait<NormalSign, CaptchaSolver, CaptchaProtocol, SignProtocol>
    for DefaultNormalOrRawSignner
where
    CaptchaProtocol: CaptchaProtocolTrait,
    SignProtocol: SignProtocolTrait,
    CaptchaSolver: CaptchaSolverTrait,
{
    type ExtData<'e> = ();

    #[inline]
    fn sign<'a, Sessions: Iterator<Item = &'a Session>>(
        &mut self,
        sign: &NormalSign,
        sessions: Sessions,
    ) -> Result<HashMap<&'a SessionUserInfo, SignResult>, SignError> {
        sign_::<CaptchaSolver, CaptchaProtocol, SignProtocol, Sessions>(sign.as_inner(), sessions)
    }

    /// 事实上不会被 [`SignnerTrait::sign`] 调用。
    #[inline]
    fn sign_single(
        sign: &NormalSign,
        session: &Session,
        _: Self::ExtData<'_>,
    ) -> Result<SignResult, SignError> {
        sign_single_::<CaptchaSolver, CaptchaProtocol, SignProtocol>(sign.as_inner(), session)
    }
}

impl<CaptchaSolver: CaptchaSolverTrait, CaptchaProtocol, SignProtocol>
    SignnerTrait<RawSign, CaptchaSolver, CaptchaProtocol, SignProtocol>
    for DefaultNormalOrRawSignner
where
    CaptchaProtocol: CaptchaProtocolTrait,
    SignProtocol: SignProtocolTrait,
{
    type ExtData<'e> = ();

    #[inline]
    fn sign<'a, Sessions: Iterator<Item = &'a Session>>(
        &mut self,
        sign: &RawSign,
        sessions: Sessions,
    ) -> Result<HashMap<&'a SessionUserInfo, SignResult>, SignError> {
        sign_::<CaptchaSolver, CaptchaProtocol, SignProtocol, Sessions>(sign, sessions)
    }

    /// 事实上不会被 [`SignnerTrait::sign`] 调用。
    #[inline]
    fn sign_single(
        sign: &RawSign,
        session: &Session,
        _: Self::ExtData<'_>,
    ) -> Result<SignResult, SignError> {
        sign_single_::<CaptchaSolver, CaptchaProtocol, SignProtocol>(sign, session)
    }
}
