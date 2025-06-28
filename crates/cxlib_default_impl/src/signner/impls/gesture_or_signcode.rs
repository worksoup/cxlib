use crate::sign::GestureOrSigncodeSign;
use cxlib_protocol::collect::{CaptchaProtocolTrait, SignProtocolTrait};
use cxlib_sign::{SignError, SignResult, SignTrait, SignnerTrait, utils::CaptchaSolver};
use cxlib_types::Session;
use std::collections::HashMap;

pub struct DefaultGestureOrSigncodeSignner(String);

impl DefaultGestureOrSigncodeSignner {
    pub fn new(signcode: &str) -> Self {
        Self(signcode.to_string())
    }
}

impl<CaptchaProtocol, SignProtocol>
    SignnerTrait<GestureOrSigncodeSign<SignProtocol>, CaptchaProtocol, SignProtocol>
    for DefaultGestureOrSigncodeSignner
where
    CaptchaProtocol: CaptchaProtocolTrait,
    SignProtocol: SignProtocolTrait,
{
    type ExtData<'e> = &'e str;

    fn sign<'a, U, Sessions: Iterator<Item = &'a Session<U>> + Clone>(
        &mut self,
        sign: &GestureOrSigncodeSign<SignProtocol>,
        sessions: Sessions,
        captcha_solver: &CaptchaSolver,
    ) -> Result<HashMap<&'a Session<U>, SignResult>, SignError> {
        #[allow(clippy::mutable_key_type)]
        let mut map = HashMap::new();
        for session in sessions {
            let a = <Self as SignnerTrait<
                GestureOrSigncodeSign<SignProtocol>,
                CaptchaProtocol,
                SignProtocol,
            >>::sign_single(sign, session, captcha_solver, &self.0)?;
            map.insert(session, a);
        }
        Ok(map)
    }

    fn sign_single<U>(
        sign: &GestureOrSigncodeSign<SignProtocol>,
        session: &Session<U>,
        captcha_solver: &CaptchaSolver,
        signcode: &str,
    ) -> Result<SignResult, SignError> {
        sign.pre_sign_and_sign::<CaptchaProtocol, U>(session, &(), captcha_solver, signcode)
    }
}
