use crate::sign::PhotoSign;
use cxlib_captcha::CaptchaSolverTrait;
use cxlib_error_utils::CxlibResultUtils;
use cxlib_protocol::collect::{CaptchaProtocolTrait, SignProtocolTrait, TypesProtocolTrait};
use cxlib_sign::{SignError, SignResult, SignTrait, SignnerTrait};
use cxlib_types::{Photo, Session};
use log::warn;
use std::{collections::HashMap, path::PathBuf};

pub struct DefaultPhotoSignner {
    // TODO: 改为Vec, 每一个用户对应一个图片位置。
    path: Option<PathBuf>,
}

impl DefaultPhotoSignner {
    #[inline]
    pub fn new(path: &Option<PathBuf>) -> Self {
        let path = path.as_ref().and_then(|pic| {
            std::fs::metadata(pic).ok().and_then(|metadata| {
                if metadata.is_dir() {
                    crate::utils::find_latest_pic(pic).ok()
                } else {
                    Some(pic.to_owned())
                }
            })
        });
        Self { path }
    }
}
impl<CaptchaSolver: CaptchaSolverTrait, CaptchaProtocol, SignProtocol, TypesProtocol>
    SignnerTrait<PhotoSign<TypesProtocol>, CaptchaSolver, CaptchaProtocol, SignProtocol>
    for DefaultPhotoSignner
where
    CaptchaProtocol: CaptchaProtocolTrait,
    SignProtocol: SignProtocolTrait,
    TypesProtocol: TypesProtocolTrait + 'static,
{
    type ExtData<'e> = &'e Photo<TypesProtocol>;

    fn sign<'a, U, Sessions: Iterator<Item = &'a Session<U>>>(
        &mut self,
        sign: &PhotoSign<TypesProtocol>,
        sessions: Sessions,
    ) -> Result<HashMap<&'a Session<U>, SignResult>, SignError> {
        let mut pic_map = Vec::new();
        let mut session_to_index = HashMap::new();
        let sessions = sessions.collect::<Vec<_>>();
        if let Some(pic) = self.path.as_ref() {
            // TODO: 需要测试多个用户能否用同一个photo token签到。
            for session in sessions.clone() {
                let photo = Photo::get_from_file(session, pic).log_ok();
                if let Some(photo) = photo {
                    pic_map.push(photo);
                    for session in sessions.clone() {
                        session_to_index.insert(session.uid(), 0);
                    }
                    break;
                }
            }
        } else {
            let mut index = 0;
            for session in sessions.clone() {
                let photo = Photo::default(session);
                session_to_index.insert(session.uid(), index);
                if let Some(photo) = photo {
                    pic_map.insert(index, photo);
                    index += 1;
                } else {
                    warn!(
                        "用户[{}]在拍照签到时未能获取到照片，将尝试使用其他用户的照片！",
                        session.name(),
                    );
                }
            }
        }
        #[allow(clippy::mutable_key_type)]
        let mut map = HashMap::new();
        for session in sessions {
            let index = session_to_index[session.uid()];
            if let Some(photo) = pic_map.get(index).cloned() {
                let a = <Self as SignnerTrait<
                    PhotoSign<TypesProtocol>,
                    CaptchaSolver,
                    CaptchaProtocol,
                    SignProtocol,
                >>::sign_single(sign, session, &photo)?;
                map.insert(session, a);
            } else {
                map.insert(
                    session,
                    SignResult::Failure {
                        msg: format!("拍照签到[{}]没有获取到有效的照片！", sign.as_inner().name()),
                    },
                );
            }
        }
        Ok(map)
    }

    #[inline]
    fn sign_single<U>(
        sign: &PhotoSign<TypesProtocol>,
        session: &Session<U>,
        photo: &Photo<TypesProtocol>,
    ) -> Result<SignResult, SignError> {
        sign.check_state_and_do_sign::<CaptchaSolver, CaptchaProtocol, SignProtocol, U>(
            session,
            &(),
            photo,
        )
    }
}
