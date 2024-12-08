use crate::{sign::QrCodeSign, signner::LocationInfoGetterTrait};
use cxlib_error::Error;
use cxlib_imageproc::Point;
use cxlib_sign::{SignResult, SignTrait, SignnerTrait};
use cxlib_types::Location;
use cxlib_user::Session;
use cxlib_utils::inquire_confirm;
use log::warn;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

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

impl<T: LocationInfoGetterTrait + Sync + Send> SignnerTrait<QrCodeSign>
    for DefaultQrCodeSignner<'_, T>
{
    type ExtData<'e> = (&'e str, &'e Location);

    fn sign<'a, Sessions: Iterator<Item = &'a Session> + Clone>(
        &mut self,
        sign: &mut QrCodeSign,
        sessions: Sessions,
    ) -> Result<HashMap<&'a Session, SignResult>, Error> {
        let location = self
            .location_info_getter
            .get_locations(sign.as_location_sign_mut(), self.location_str)
            .unwrap_or_else(|| {
                warn!("未获取到位置信息，请检查位置列表或检查输入。");
                Location::get_none_location()
            });
        #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
        let enc = Self::enc_gen(sign, self.path, self.enc, self.precisely)?;
        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        let enc = Self::enc_gen(self.path, self.enc)?;
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
                let enc = enc.clone();
                let location = location.clone();
                let h = std::thread::spawn(move || {
                    let a = Self::sign_single(&mut sign, &session, (&enc, &location))
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
                let state = Self::sign_single(sign, session, (&enc, &location))?;
                map.insert(session, state);
            }
        }
        Ok(map)
    }

    fn sign_single(
        sign: &mut QrCodeSign,
        session: &Session,
        (enc, location): (&str, &Location),
    ) -> Result<SignResult, Error> {
        sign.pre_sign_and_sign(session, enc, location)
    }
}

impl<'a, T: LocationInfoGetterTrait> DefaultQrCodeSignner<'a, T> {
    fn pic_to_enc(pic: &PathBuf) -> Result<String, Error> {
        if std::fs::metadata(pic).expect("图片路径出错。").is_dir() {
            loop {
                let yes = inquire_confirm("二维码图片是否就绪？", "本程序会读取 `--pic` 参数所指定的路径下最新修改的图片。你可以趁现在获取这张图片，然后按下回车进行签到。");
                if yes {
                    break;
                }
            }
            let pic = crate::utils::find_latest_pic(pic)?;
            Self::pic_path_to_qrcode_result(pic.to_str().unwrap()).ok_or_else(|| {
                Error::EncError(
                    "未能识别到二维码，可能是二维码模糊、过小等，请确保图片易于识别。".to_owned(),
                )
            })
        } else if let Some(enc) = Self::pic_path_to_qrcode_result(pic.to_str().unwrap()) {
            Ok(enc)
        } else {
            return Err(Error::EncError("二维码中没有 `enc` 参数！".to_owned()));
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    pub fn enc_gen(path: &Option<PathBuf>, enc: &Option<String>) -> Result<String, Error> {
        let enc = if let Some(enc) = enc {
            enc.clone()
        } else if let Some(pic) = path {
            pic_to_enc(pic)?
        } else {
            return Err(Error::EncError("未获取到 `enc` 参数！".to_owned()));
        };
        Ok(enc)
    }

    pub fn is_enc_qrcode_url(url: &str) -> bool {
        url.contains(&*cxlib_protocol::ProtocolItem::QrcodePat.to_string()) && url.contains("&enc=")
    }
    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    pub fn capture_screen_for_enc(is_refresh: bool, precise: bool) -> Option<String> {
        fn get_rect_contains_vertex(vertex: &[rxing::Point]) -> (Point<u32>, Point<u32>) {
            let (lt, rb) =
                cxlib_imageproc::get_rect_contains_vertex(vertex.iter().map(|vertex| Point {
                    x: vertex.x as _,
                    y: vertex.y as _,
                }));
            let wh = rb - lt;
            (lt - Point { x: 10, y: 10 }, wh + Point { x: 20, y: 20 })
        }
        use log::{debug, error, info, warn};
        let screens = xcap::Monitor::all().unwrap_or_else(|e| panic!("{e:?}"));
        // 在所有屏幕中寻找。
        if !precise
            && is_refresh
            && !inquire_confirm(
                "二维码图片是否就绪？",
                "本程序将在屏幕上寻找签到二维码，待二维码刷新后按下回车进行签到。",
            )
        {
            return None;
        }
        for screen in screens {
            // 先截取整个屏幕。
            let pic = screen.capture_image().unwrap_or_else(|e| {
                error!("{e:?}");
                panic!("{e:?}")
            });
            info!("已截屏。");
            // 如果成功识别到二维码。
            let results = Self::scan_qrcode(image::DynamicImage::from(pic), &mut HashMap::new());
            let results = if let Ok(results) = results {
                results
            } else {
                continue;
            };
            // 在结果中寻找。
            for r in &results {
                let url = r.getText();
                // 如果符合要求的二维码。
                if !Self::is_enc_qrcode_url(url) {
                    warn!("{url:?}不是有效的签到二维码！");
                    continue;
                }
                info!("存在签到二维码。");
                return if precise && is_refresh && inquire_confirm("二维码图片是否就绪？", "本程序已在屏幕上找到签到二维码。请不要改变该二维码的位置，待二维码刷新后按下回车进行签到。") {
                    // 如果是定时刷新的二维码，等待二维码刷新。
                    let qrcode_pos_on_screen = get_rect_contains_vertex(r.getPoints());
                    debug!("二维码位置：{:?}", qrcode_pos_on_screen);
                    let pic = screen
                        .capture_image()
                        .unwrap_or_else(|e| panic!("{e:?}"));
                    let cut_pic =cxlib_imageproc::cut_picture(&pic, qrcode_pos_on_screen.0, qrcode_pos_on_screen.1);
                    let r = Self::scan_qrcode(cut_pic.to_image().into(), &mut HashMap::new()).unwrap_or_else(|e| panic!("{e:?}"));
                    Self::find_qrcode_sign_enc_in_url(r[0].getText())
                } else {
                    // 如果不是精确截取的二维码，则不需要提示。
                    Self::find_qrcode_sign_enc_in_url(url)
                };
            }
        }
        None
    }
    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    pub fn enc_gen(
        sign: &crate::sign::QrCodeSign,
        path: &Option<PathBuf>,
        enc: &Option<String>,
        precisely: bool,
    ) -> Result<String, Error> {
        let enc = if let Some(enc) = enc {
            enc.clone()
        } else if let Some(pic) = path {
            Self::pic_to_enc(pic)?
        } else if let Some(enc) = Self::capture_screen_for_enc(sign.is_refresh(), precisely) {
            enc
        } else {
            return Err(Error::EncError("截屏时未获取到 `enc` 参数！".to_owned()));
        };
        Ok(enc)
    }
    pub fn find_qrcode_sign_enc_in_url(url: &str) -> Option<String> {
        // 在二维码图片中会有一个参数 `c`, 二维码预签到时需要。
        // 但是该参数似乎暂时可以从 `signDetail` 接口获取到。所以此处先注释掉。
        // let beg = r.find("&c=").unwrap();
        // let c = &r[beg + 3..beg + 9];
        // (c.to_owned(), enc.to_owned())
        // 有时二维码里没有参数，原因不明。
        let r = url
            .find("&enc=")
            .map(|beg| url[beg + 5..beg + 37].to_owned());
        if r.is_none() {
            log::warn!("{url:?}中没有找到二维码！");
        }
        r
    }
    pub fn pic_path_to_qrcode_result(pic_path: &str) -> Option<String> {
        let r = Self::scan_file(pic_path).ok()?;
        Self::find_qrcode_sign_enc_in_url(r.first()?.getText())
    }
    pub fn scan_qrcode(
        image: image::DynamicImage,
        hints: &mut rxing::DecodingHintDictionary,
    ) -> rxing::common::Result<Vec<rxing::RXingResult>> {
        hints
            .entry(rxing::DecodeHintType::TRY_HARDER)
            .or_insert(rxing::DecodeHintValue::TryHarder(true));
        let reader = rxing::MultiFormatReader::default();
        let mut scanner = rxing::multi::GenericMultipleBarcodeReader::new(reader);
        rxing::multi::MultipleBarcodeReader::decode_multiple_with_hints(
            &mut scanner,
            &mut rxing::BinaryBitmap::new(rxing::common::HybridBinarizer::new(
                rxing::BufferedImageLuminanceSource::new(image),
            )),
            hints,
        )
    }

    pub fn scan_file(pic_path: &str) -> rxing::common::Result<Vec<rxing::RXingResult>> {
        rxing::helpers::detect_multiple_in_file(pic_path)
    }
}
