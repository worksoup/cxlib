use cxlib_error::Error;
use cxlib_qrcode_utils::find_qrcode_sign_enc_in_url;
use cxlib_utils::*;
use std::path::PathBuf;

fn pic_to_enc(pic: &PathBuf) -> Result<String, Error> {
    if std::fs::metadata(pic).expect("图片路径出错。").is_dir() {
        pic_dir_or_path_to_pic_path(pic)?
            .and_then(|pic| pic_path_to_qrcode_result(pic.to_str().unwrap()))
            .ok_or_else(|| {
                Error::EncError("图片文件夹下没有图片（`png` 或 `jpg` 文件）！".to_owned())
            })
    } else if let Some(enc) = pic_path_to_qrcode_result(pic.to_str().unwrap()) {
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
        pic_to_enc(pic)?
    } else if let Some(enc) =
        cxlib_qrcode_utils::capture_screen_for_enc(sign.is_refresh(), precisely)
    {
        enc
    } else {
        return Err(Error::EncError("截屏时未获取到 `enc` 参数！".to_owned()));
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
    let r = cxlib_qrcode_utils::scan_file(pic_path).ok()?;
    find_qrcode_sign_enc_in_url(r.first()?.getText())
}
