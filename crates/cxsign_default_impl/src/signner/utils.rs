use cxsign_activity::sign::QrCodeSign;
use cxsign_error::Error;
use cxsign_store::{DataBase, DataBaseTableTrait};
use cxsign_types::{Location, LocationTable};
use cxsign_utils::*;
use log::warn;
use std::path::PathBuf;

pub fn enc_gen(
    sign: &QrCodeSign,
    path: &Option<PathBuf>,
    enc: &Option<String>,
    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))] precisely: bool,
) -> Result<String, Error> {
    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    let enc = if let Some(enc) = enc {
        enc.clone()
    } else if let Some(pic) = path {
        if std::fs::metadata(pic).unwrap().is_dir() {
            if let Some(pic) = pic_dir_or_path_to_pic_path(pic)?
                && let Some(enc) = pic_path_to_qrcode_result(pic.to_str().unwrap())
            {
                enc
            } else {
                return Err(Error::EncError(
                    "图片文件夹下没有图片（`png` 或 `jpg` 文件）！".to_owned(),
                ));
            }
        } else if let Some(enc) = pic_path_to_qrcode_result(pic.to_str().unwrap()) {
            enc
        } else {
            return Err(Error::EncError("二维码中没有 `enc` 参数！".to_owned()));
        }
    } else if let Some(enc) =
        cxsign_qrcode_utils::capture_screen_for_enc(sign.is_refresh(), precisely)
    {
        enc
    } else {
        return Err(Error::EncError("截屏时未获取到 `enc` 参数！".to_owned()));
    };

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    let enc = if let Some(enc) = enc {
        enc.clone()
    } else if let Some(pic) = path {
        if std::fs::metadata(pic).unwrap().is_dir() {
            if let Some(pic) = crate::utils::pic_dir_or_path_to_pic_path(pic)?
                && let Some(enc) = crate::utils::pic_path_to_qrcode_result(pic.to_str().unwrap())
            {
                enc
            } else {
                return Err(Error::EncError(
                    "图片文件夹下没有图片（`png` 或 `jpg` 文件）！".to_owned(),
                ));
            }
        } else if let Some(enc) = crate::utils::pic_path_to_qrcode_result(pic.to_str().unwrap()) {
            enc
        } else {
            return Err(Error::EncError("二维码中没有 `enc` 参数！".to_owned()));
        }
    } else {
        return Err(Error::EncError("未获取到 `enc` 参数！".to_owned()));
    };
    Ok(enc)
}
pub fn pic_dir_or_path_to_pic_path(pic_dir: &PathBuf) -> Result<Option<PathBuf>, std::io::Error> {
    loop {
        let yes = inquire_confirm("二维码图片是否就绪？", "本程序会读取 `--pic` 参数所指定的路径下最新修改的图片。你可以趁现在获取这张图片，然后按下回车进行签到。");
        if yes {
            break;
        }
    }
    let pic_path = {
        let pic_dir = std::fs::read_dir(pic_dir)?;
        let mut all_files_in_dir = Vec::new();
        for k in pic_dir {
            let k = k?;
            let file_type = k.file_type()?;
            if file_type.is_file() && {
                let file_name = k.file_name();
                file_name.to_str().is_some_and(|file_name| {
                    file_name
                        .split('.')
                        .last()
                        .is_some_and(|file_ext| file_ext == "png" || file_ext == "jpg")
                })
            } {
                all_files_in_dir.push(k);
            }
        }

        all_files_in_dir.sort_by(|a, b| {
            b.metadata()
                .unwrap()
                .modified()
                .unwrap()
                .cmp(&a.metadata().unwrap().modified().unwrap())
        });
        all_files_in_dir.first().map(|d| d.path())
    };
    Ok(pic_path)
}

pub fn pic_path_to_qrcode_result(pic_path: &str) -> Option<String> {
    let r = cxsign_qrcode_utils::scan_file(pic_path).ok()?;
    find_qrcode_sign_enc_in_url(r.first()?.getText())
}

pub fn location_str_to_location(
    db: &DataBase,
    location_str: &Option<String>,
) -> Result<Location, String> {
    let table = LocationTable::from_ref(db);
    if let Some(ref location_str) = location_str {
        let location_str = location_str.trim();
        if let Ok(location) = location_str.parse() {
            Ok(location)
        } else if let Some(location) = table.get_location_by_alias(location_str) {
            Ok(location)
        } else if let Ok(location_id) = location_str.parse() {
            if table.has_location(location_id) {
                let (_, location) = table.get_location(location_id);
                Ok(location)
            } else {
                Err(location_str.to_owned())
            }
        } else {
            Err(location_str.to_owned())
        }
    } else {
        warn!("位置字符串不存在！");
        Err("".to_string())
    }
}
