use crate::sign::NormalSign;
use cxlib_captcha::CaptchaSolverTrait;
use cxlib_protocol::collect::{CaptchaProtocolTrait, SignProtocolTrait};
use cxlib_sign::{AsRaw, SignError, SignResult, SignTrait, SignnerTrait};
use cxlib_types::{RawSign, Session};
use std::collections::HashMap;

pub struct DefaultNormalOrRawSignner;

#[inline]
fn sign_single_<CaptchaSolver: CaptchaSolverTrait, CaptchaProtocol, SignProtocol, U>(
    sign: &RawSign,
    session: &Session<U>,
) -> Result<SignResult, SignError>
where
    CaptchaProtocol: CaptchaProtocolTrait,
    SignProtocol: SignProtocolTrait,
{
    sign.check_state_and_do_sign::<CaptchaSolver, CaptchaProtocol, SignProtocol, U>(
        session,
        &(),
        &(),
    )
}
#[inline]
fn sign_<
    'a,
    CaptchaSolver: CaptchaSolverTrait,
    CaptchaProtocol,
    SignProtocol,
    U,
    Sessions: Iterator<Item = &'a Session<U>>,
>(
    sign: &RawSign,
    sessions: Sessions,
) -> Result<HashMap<&'a Session<U>, SignResult>, SignError>
where
    CaptchaProtocol: CaptchaProtocolTrait,
    SignProtocol: SignProtocolTrait,
{
    #[allow(clippy::mutable_key_type)]
    let mut map = HashMap::new();
    for session in sessions {
        let a = sign_single_::<CaptchaSolver, CaptchaProtocol, SignProtocol, U>(sign, session)?;
        map.insert(session, a);
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
    fn sign<'a, U, Sessions: Iterator<Item = &'a Session<U>>>(
        &mut self,
        sign: &NormalSign,
        sessions: Sessions,
    ) -> Result<HashMap<&'a Session<U>, SignResult>, SignError> {
        sign_::<CaptchaSolver, CaptchaProtocol, SignProtocol, U, Sessions>(
            sign.as_inner(),
            sessions,
        )
    }

    /// 事实上不会被 [`SignnerTrait::sign`] 调用。
    #[inline]
    fn sign_single<U>(
        sign: &NormalSign,
        session: &Session<U>,
        _: Self::ExtData<'_>,
    ) -> Result<SignResult, SignError> {
        sign_single_::<CaptchaSolver, CaptchaProtocol, SignProtocol, U>(sign.as_inner(), session)
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
    fn sign<'a, U, Sessions: Iterator<Item = &'a Session<U>>>(
        &mut self,
        sign: &RawSign,
        sessions: Sessions,
    ) -> Result<HashMap<&'a Session<U>, SignResult>, SignError> {
        sign_::<CaptchaSolver, CaptchaProtocol, SignProtocol, U, Sessions>(sign, sessions)
    }

    /// 事实上不会被 [`SignnerTrait::sign`] 调用。
    #[inline]
    fn sign_single<U>(
        sign: &RawSign,
        session: &Session<U>,
        _: Self::ExtData<'_>,
    ) -> Result<SignResult, SignError> {
        sign_single_::<CaptchaSolver, CaptchaProtocol, SignProtocol, U>(sign, session)
    }
}
