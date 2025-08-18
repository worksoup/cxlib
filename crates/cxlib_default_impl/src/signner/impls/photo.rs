use crate::sign::PhotoSign;
use cxlib_captcha::CaptchaSolverTrait;
use cxlib_protocol::collect::{CaptchaProtocolTrait, NetdiskProtocolTrait, SignProtocolTrait};
use cxlib_sign::{AsRaw, SignError, SignResult, SignTrait, SignnerTrait};
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
impl<CaptchaSolver: CaptchaSolverTrait, CaptchaProtocol, SignProtocol, NetdiskProtocol>
    SignnerTrait<PhotoSign<NetdiskProtocol>, CaptchaSolver, CaptchaProtocol, SignProtocol>
    for DefaultPhotoSignner
where
    CaptchaProtocol: CaptchaProtocolTrait,
    SignProtocol: SignProtocolTrait,
    NetdiskProtocol: NetdiskProtocolTrait + 'static,
{
    type ExtData<'e> = &'e Photo<NetdiskProtocol>;

    fn sign<'a, U, Sessions: Iterator<Item = &'a Session<U>>>(
        &mut self,
        sign: &PhotoSign<NetdiskProtocol>,
        sessions: Sessions,
    ) -> Result<HashMap<&'a Session<U>, SignResult>, SignError> {
        let mut pic_map = Vec::new();
        let mut session_to_index = HashMap::new();
        let sessions = sessions.collect::<Vec<_>>();
        #[cfg(not(feature = "upload-file"))]
        {
            warn!("未启用上传文件支持。");
        }
        if cfg!(feature = "upload-file")
            && let Some(pic) = self.path.as_ref()
        {
            #[cfg(not(feature = "upload-file"))]
            {
                let _ = pic;
                unreachable!("未启用上传文件支持。");
            }
            // TODO: 需要测试多个用户能否用同一个photo token签到。
            #[cfg(feature = "upload-file")]
            for session in sessions.clone() {
                use cxlib_error_utils::CxlibResultUtils;
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
                    PhotoSign<NetdiskProtocol>,
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
        sign: &PhotoSign<NetdiskProtocol>,
        session: &Session<U>,
        photo: &Photo<NetdiskProtocol>,
    ) -> Result<SignResult, SignError> {
        sign.check_state_and_do_sign::<CaptchaSolver, CaptchaProtocol, SignProtocol, U>(
            session,
            &(),
            photo,
        )
    }
}
