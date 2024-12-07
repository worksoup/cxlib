use crate::sign::{GestureSign, SigncodeSign};
use cxlib_error::Error;
use cxlib_sign::{SignResult, SignTrait, SignnerTrait};
use cxlib_user::Session;
use std::collections::HashMap;

pub struct DefaultGestureOrSigncodeSignner(String);

impl DefaultGestureOrSigncodeSignner {
    pub fn new(signcode: &str) -> Self {
        Self(signcode.to_string())
    }
}

impl SignnerTrait<GestureSign> for DefaultGestureOrSigncodeSignner {
    type ExtData<'e> = &'e str;

    fn sign<'a, Sessions: Iterator<Item = &'a Session> + Clone>(
        &mut self,
        sign: &mut GestureSign,
        sessions: Sessions,
    ) -> Result<HashMap<&'a Session, SignResult>, Error> {
        #[allow(clippy::mutable_key_type)]
        let mut map = HashMap::new();
        for session in sessions {
            let a = Self::sign_single(sign, session, &self.0)?;
            map.insert(session, a);
        }
        Ok(map)
    }

    fn sign_single(
        sign: &mut GestureSign,
        session: &Session,
        signcode: &str,
    ) -> Result<SignResult, Error> {
        sign.pre_sign_and_sign(session, &(), signcode)
    }
}

impl SignnerTrait<SigncodeSign> for DefaultGestureOrSigncodeSignner {
    type ExtData<'e> = &'e str;

    fn sign<'a, Sessions: Iterator<Item = &'a Session> + Clone>(
        &mut self,
        sign: &mut SigncodeSign,
        sessions: Sessions,
    ) -> Result<HashMap<&'a Session, SignResult>, Error> {
        #[allow(clippy::mutable_key_type)]
        let mut map = HashMap::new();
        for session in sessions {
            let a = Self::sign_single(sign, session, &self.0)?;
            map.insert(session, a);
        }
        Ok(map)
    }

    fn sign_single(
        sign: &mut SigncodeSign,
        session: &Session,
        gesture: &str,
    ) -> Result<SignResult, Error> {
        sign.pre_sign_and_sign(session, &(), gesture)
    }
}
