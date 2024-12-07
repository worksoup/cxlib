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
    type ExtData<'e> = ();

    fn sign<'a, Sessions: Iterator<Item = &'a Session> + Clone>(
        &mut self,
        sign: &mut GestureSign,
        sessions: Sessions,
    ) -> Result<HashMap<&'a Session, SignResult>, Error> {
        #[allow(clippy::mutable_key_type)]
        let mut map = HashMap::new();
        for session in sessions {
            let a = self.sign_single(sign, session, ())?;
            map.insert(session, a);
        }
        Ok(map)
    }

    fn sign_single(
        &mut self,
        sign: &mut GestureSign,
        session: &Session,
        _: Self::ExtData<'_>,
    ) -> Result<SignResult, Error> {
        sign.pre_sign_and_sign(session, &(), &self.0)
    }
}

impl SignnerTrait<SigncodeSign> for DefaultGestureOrSigncodeSignner {
    type ExtData<'e> = ();

    fn sign<'a, Sessions: Iterator<Item = &'a Session> + Clone>(
        &mut self,
        sign: &mut SigncodeSign,
        sessions: Sessions,
    ) -> Result<HashMap<&'a Session, SignResult>, Error> {
        #[allow(clippy::mutable_key_type)]
        let mut map = HashMap::new();
        for session in sessions {
            let a = self.sign_single(sign, session, ())?;
            map.insert(session, a);
        }
        Ok(map)
    }

    fn sign_single(
        &mut self,
        sign: &mut SigncodeSign,
        session: &Session,
        _: Self::ExtData<'_>,
    ) -> Result<SignResult, Error> {
        sign.pre_sign_and_sign(session, &(), &self.0)
    }
}
