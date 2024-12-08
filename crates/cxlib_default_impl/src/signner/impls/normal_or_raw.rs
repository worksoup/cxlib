use crate::sign::NormalSign;
use cxlib_activity::RawSign;
use cxlib_error::SignError;
use cxlib_sign::{SignResult, SignTrait, SignnerTrait};
use cxlib_user::Session;
use std::collections::HashMap;

pub struct DefaultNormalOrRawSignner;

fn sign_single_(sign: &RawSign, session: &Session) -> Result<SignResult, SignError> {
    sign.pre_sign_and_sign(session, &(), &())
}
fn sign_<'a, Sessions: Iterator<Item = &'a Session> + Clone>(
    sign: &RawSign,
    sessions: Sessions,
) -> Result<HashMap<&'a Session, SignResult>, SignError> {
    #[allow(clippy::mutable_key_type)]
    let mut map = HashMap::new();
    for session in sessions {
        let a = sign_single_(sign, session)?;
        map.insert(session, a);
    }
    Ok(map)
}

impl SignnerTrait<NormalSign> for DefaultNormalOrRawSignner {
    type ExtData<'e> = ();

    fn sign<'a, Sessions: Iterator<Item = &'a Session> + Clone>(
        &mut self,
        sign: &mut NormalSign,
        sessions: Sessions,
    ) -> Result<HashMap<&'a Session, SignResult>, SignError> {
        sign_(sign.as_inner(), sessions)
    }

    /// 事实上不会被 [`SignnerTrait::sign`] 调用。
    fn sign_single(
        sign: &mut NormalSign,
        session: &Session,
        _: Self::ExtData<'_>,
    ) -> Result<SignResult, SignError> {
        sign_single_(sign.as_inner(), session)
    }
}

impl SignnerTrait<RawSign> for DefaultNormalOrRawSignner {
    type ExtData<'e> = ();

    fn sign<'a, Sessions: Iterator<Item = &'a Session> + Clone>(
        &mut self,
        sign: &mut RawSign,
        sessions: Sessions,
    ) -> Result<HashMap<&'a Session, SignResult>, SignError> {
        sign_(sign, sessions)
    }

    /// 事实上不会被 [`SignnerTrait::sign`] 调用。
    fn sign_single(
        sign: &mut RawSign,
        session: &Session,
        _: Self::ExtData<'_>,
    ) -> Result<SignResult, SignError> {
        sign_single_(sign, session)
    }
}
