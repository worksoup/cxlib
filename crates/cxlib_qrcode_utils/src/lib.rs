use cxlib_protocol::ProtocolItem;
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
pub use desktop_lwm::*;
use log::warn;
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
mod desktop_lwm {
    use crate::{find_qrcode_sign_enc_in_url, is_enc_qrcode_url, scan_qrcode};
    use cxlib_utils::inquire_confirm;
    use log::{debug, error, info, warn};
    use rxing::Point;
    use std::collections::HashMap;

    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    fn get_rect_contains_vertex(
        vertex: &[Point],
    ) -> (cxlib_imageproc::Point<u32>, cxlib_imageproc::Point<u32>) {
        let (lt, rb) = cxlib_imageproc::get_rect_contains_vertex(vertex.iter().map(|vertex| {
            cxlib_imageproc::Point {
                x: vertex.x as _,
                y: vertex.y as _,
            }
        }));
        let wh = rb - lt;
        (
            lt - cxlib_imageproc::Point { x: 10, y: 10 },
            wh + cxlib_imageproc::Point { x: 20, y: 20 },
        )
    }
    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    pub fn capture_screen_for_enc(is_refresh: bool, precise: bool) -> Option<String> {
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
            let results = scan_qrcode(xcap::image::DynamicImage::from(pic), &mut HashMap::new());
            let results = if let Ok(results) = results {
                results
            } else {
                continue;
            };
            // 在结果中寻找。
            for r in &results {
                let url = r.getText();
                // 如果符合要求的二维码。
                if !is_enc_qrcode_url(url) {
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
                    let cut_pic = cxlib_imageproc::cut_picture(&pic, qrcode_pos_on_screen.0, qrcode_pos_on_screen.1);
                    let r = scan_qrcode(cut_pic.to_image().into(), &mut HashMap::new()).unwrap_or_else(|e| panic!("{e:?}"));
                    find_qrcode_sign_enc_in_url(r[0].getText())
                } else {
                    // 如果不是精确截取的二维码，则不需要提示。
                    find_qrcode_sign_enc_in_url(url)
                };
            }
        }
        None
    }
}
pub fn is_enc_qrcode_url(url: &str) -> bool {
    url.contains(&*ProtocolItem::QrcodePat.to_string()) && url.contains("&enc=")
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
        warn!("{url:?}中没有找到二维码！")
    }
    r
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
