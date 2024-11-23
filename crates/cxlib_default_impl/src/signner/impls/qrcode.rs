use crate::sign::QrCodeSign;
use crate::signner::LocationInfoGetterTrait;
use cxlib_error::Error;
use cxlib_sign::{SignResult, SignTrait};
use cxlib_signner::SignnerTrait;
use cxlib_user::Session;
use log::warn;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct DefaultQrCodeSignner<'a, T: LocationInfoGetterTrait> {
    location_info_getter: T,
    location_str: &'a Option<String>,
    path: &'a Option<PathBuf>,
    enc: &'a Option<String>,
    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    precisely: bool,
}
impl<'a, T: LocationInfoGetterTrait> DefaultQrCodeSignner<'a, T> {
    pub fn new(
        location_info_getter: T,
        location_str: &'a Option<String>,
        path: &'a Option<PathBuf>,
        enc: &'a Option<String>,
        #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
        precisely: bool,
    ) -> Self {
        Self {
            location_info_getter,
            location_str,
            path,
            enc,
            #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
            precisely,
        }
    }
}

impl<T: LocationInfoGetterTrait> SignnerTrait<QrCodeSign> for DefaultQrCodeSignner<'_, T> {
    type ExtData<'e> = ();

    fn sign<'a, Sessions: Iterator<Item = &'a Session> + Clone>(
        &mut self,
        sign: &mut QrCodeSign,
        sessions: Sessions,
    ) -> Result<HashMap<&'a Session, SignResult>, Error> {
        let location = self
            .location_info_getter
            .get_locations(sign.as_location_sign_mut(), self.location_str);
        if let Some(location) = location {
            sign.set_location(location);
        } else {
            warn!("未获取到位置信息，请检查位置列表或检查输入。");
        }
        #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
        let enc = crate::utils::enc_gen(sign, self.path, self.enc, self.precisely)?;
        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        let enc = crate::utils::enc_gen(sign, self.path, self.enc)?;
        sign.set_enc(enc);
        #[allow(clippy::mutable_key_type)]
        let mut map = HashMap::new();
        if sign.is_refresh() {
            let sessions = sessions.collect::<Vec<&'a Session>>();
            let index_result_map = Arc::new(Mutex::new(HashMap::new()));
            let mut handles = Vec::new();
            for (sessions_index, session) in sessions.clone().into_iter().enumerate() {
                let index_result_map = Arc::clone(&index_result_map);
                let mut sign = sign.clone();
                let session = session.clone();
                let h = std::thread::spawn(move || {
                    let a = Self::sign_single(&mut sign, &session, ())
                        .unwrap_or_else(|e| SignResult::Fail { msg: e.to_string() });
                    index_result_map.lock().unwrap().insert(sessions_index, a);
                });
                handles.push(h);
            }
            for h in handles {
                h.join().unwrap();
            }
            for (i, r) in Arc::into_inner(index_result_map)
                .unwrap()
                .into_inner()
                .unwrap()
            {
                map.insert(sessions[i], r);
            }
        } else {
            for session in sessions {
                let state = Self::sign_single(sign, session, ())?;
                map.insert(session, state);
            }
        }
        Ok(map)
    }

    fn sign_single(sign: &mut QrCodeSign, session: &Session, _: ()) -> Result<SignResult, Error> {
        let r = sign.pre_sign(session).map_err(Error::from)?;
        unsafe { sign.sign_unchecked(session, r) }.map_err(Error::from)
    }
}
