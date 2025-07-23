use crate::sign::NormalSign;
use cxlib_captcha::CaptchaSolver;
use cxlib_protocol::collect::{CaptchaProtocolTrait, SignProtocolTrait};
use cxlib_sign::{SignError, SignResult, SignTrait, SignnerTrait};
use cxlib_types::{RawSign, Session};
use std::collections::HashMap;

pub struct DefaultNormalOrRawSignner;

fn sign_single_<CaptchaProtocol, SignProtocol, U>(
    sign: &RawSign<SignProtocol>,
    session: &Session<U>,
    captcha_solver: &CaptchaSolver,
) -> Result<SignResult, SignError>
where
    CaptchaProtocol: CaptchaProtocolTrait,
    SignProtocol: SignProtocolTrait,
{
    sign.pre_sign_and_sign::<CaptchaProtocol, U>(session, &(), captcha_solver, &())
}
fn sign_<'a, CaptchaProtocol, SignProtocol, U, Sessions: Iterator<Item = &'a Session<U>> + Clone>(
    sign: &RawSign<SignProtocol>,
    sessions: Sessions,
    captcha_solver: &CaptchaSolver,
) -> Result<HashMap<&'a Session<U>, SignResult>, SignError>
where
    CaptchaProtocol: CaptchaProtocolTrait,
    SignProtocol: SignProtocolTrait,
{
    #[allow(clippy::mutable_key_type)]
    let mut map = HashMap::new();
    for session in sessions {
        let a = sign_single_::<CaptchaProtocol, SignProtocol, U>(sign, session, captcha_solver)?;
        map.insert(session, a);
    }
    Ok(map)
}

impl<CaptchaProtocol, SignProtocol>
    SignnerTrait<NormalSign<SignProtocol>, CaptchaProtocol, SignProtocol>
    for DefaultNormalOrRawSignner
where
    CaptchaProtocol: CaptchaProtocolTrait,
    SignProtocol: SignProtocolTrait,
{
    type ExtData<'e> = ();

    fn sign<'a, U, Sessions: Iterator<Item = &'a Session<U>> + Clone>(
        &mut self,
        sign: &NormalSign<SignProtocol>,
        sessions: Sessions,
        captcha_solver: &CaptchaSolver,
    ) -> Result<HashMap<&'a Session<U>, SignResult>, SignError> {
        sign_::<CaptchaProtocol, SignProtocol, U, Sessions>(
            sign.as_inner(),
            sessions,
            captcha_solver,
        )
    }

    /// 事实上不会被 [`SignnerTrait::sign`] 调用。
    fn sign_single<U>(
        sign: &NormalSign<SignProtocol>,
        session: &Session<U>,
        captcha_solver: &CaptchaSolver,
        _: Self::ExtData<'_>,
    ) -> Result<SignResult, SignError> {
        sign_single_::<CaptchaProtocol, SignProtocol, U>(sign.as_inner(), session, captcha_solver)
    }
}

impl<CaptchaProtocol, SignProtocol>
    SignnerTrait<RawSign<SignProtocol>, CaptchaProtocol, SignProtocol> for DefaultNormalOrRawSignner
where
    CaptchaProtocol: CaptchaProtocolTrait,
    SignProtocol: SignProtocolTrait,
{
    type ExtData<'e> = ();

    fn sign<'a, U, Sessions: Iterator<Item = &'a Session<U>> + Clone>(
        &mut self,
        sign: &RawSign<SignProtocol>,
        sessions: Sessions,
        captcha_solver: &CaptchaSolver,
    ) -> Result<HashMap<&'a Session<U>, SignResult>, SignError> {
        sign_::<CaptchaProtocol, SignProtocol, U, Sessions>(sign, sessions, captcha_solver)
    }

    /// 事实上不会被 [`SignnerTrait::sign`] 调用。
    fn sign_single<U>(
        sign: &RawSign<SignProtocol>,
        session: &Session<U>,
        captcha_solver: &CaptchaSolver,
        _: Self::ExtData<'_>,
    ) -> Result<SignResult, SignError> {
        sign_single_::<CaptchaProtocol, SignProtocol, U>(sign, session, captcha_solver)
    }
}
