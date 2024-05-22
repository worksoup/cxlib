use cxsign_error::Error;
use cxsign_sign::{SignResult, SignTrait};
use cxsign_user::Session;
use std::collections::HashMap;

pub trait SignnerTrait<T: SignTrait> {
    type ExtData<'e>;
    fn sign<'a, Sessions: Iterator<Item = &'a Session> + Clone>(
        &mut self,
        sign: &mut T,
        sessions: Sessions,
    ) -> Result<HashMap<&'a Session, SignResult>, Error>;
    fn sign_single(
        sign: &mut T,
        session: &Session,
        extra_data: Self::ExtData<'_>,
    ) -> Result<SignResult, Error>;
}