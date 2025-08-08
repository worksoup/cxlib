use crate::sign::GestureOrSigncodeSign;
use cxlib_captcha::CaptchaSolverTrait;
use cxlib_protocol::collect::{CaptchaProtocolTrait, SignProtocolTrait};
use cxlib_sign::{SignError, SignResult, SignTrait, SignnerTrait};
use cxlib_types::Session;
use std::collections::HashMap;

pub struct DefaultGestureOrSigncodeSignner(String);

impl DefaultGestureOrSigncodeSignner {
    pub fn new(signcode: &str) -> Self {
        Self(signcode.to_string())
    }
}

impl<CaptchaSolver: CaptchaSolverTrait, CaptchaProtocol, SignProtocol>
    SignnerTrait<GestureOrSigncodeSign, CaptchaSolver, CaptchaProtocol, SignProtocol>
    for DefaultGestureOrSigncodeSignner
where
    CaptchaProtocol: CaptchaProtocolTrait,
    SignProtocol: SignProtocolTrait,
{
    type ExtData<'e> = &'e str;

    #[inline]
    fn sign<'a, U, Sessions: Iterator<Item = &'a Session<U>>>(
        &mut self,
        sign: &GestureOrSigncodeSign,
        sessions: Sessions,
    ) -> Result<HashMap<&'a Session<U>, SignResult>, SignError> {
        #[allow(clippy::mutable_key_type)]
        let mut map = HashMap::new();
        for session in sessions {
            let a = <Self as SignnerTrait<
                GestureOrSigncodeSign,
                CaptchaSolver,
                CaptchaProtocol,
                SignProtocol,
            >>::sign_single(sign, session, &self.0)?;
            map.insert(session, a);
        }
        Ok(map)
    }

    #[inline]
    fn sign_single<U>(
        sign: &GestureOrSigncodeSign,
        session: &Session<U>,
        signcode: &str,
    ) -> Result<SignResult, SignError> {
        sign.check_state_and_do_sign::<CaptchaSolver, CaptchaProtocol, SignProtocol, U>(
            session,
            &(),
            signcode,
        )
    }
}
