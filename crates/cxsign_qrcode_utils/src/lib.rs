use cxsign_protocol::ProtocolItem;
use cxsign_utils::inquire_confirm;
use log::{debug, error, info, warn};
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
use rxing::Point;
use std::collections::HashMap;

pub fn is_enc_qrcode_url(url: &str) -> bool {
    url.contains(&*ProtocolItem::QrcodePat.to_string()) && url.contains("&enc=")
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

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
fn get_rect_contains_vertex(
    vertex: &[Point],
) -> (cxsign_imageproc::Point<u32>, cxsign_imageproc::Point<u32>) {
    let (lt, rb) = cxsign_imageproc::get_rect_contains_vertex(vertex.iter().map(|vertex| {
        cxsign_imageproc::Point {
            x: vertex.x as _,
            y: vertex.y as _,
        }
    }));
    let wh = rb - lt;
    (
        lt - cxsign_imageproc::Point { x: 10, y: 10 },
        wh + cxsign_imageproc::Point { x: 20, y: 20 },
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
                let cut_pic = cxsign_imageproc::cut_picture(&pic, qrcode_pos_on_screen.0, qrcode_pos_on_screen.1);
                let r = scan_qrcode(cut_pic.to_image().into(), &mut HashMap::new()).unwrap_or_else(|e| panic!("{e:?}"));
                cxsign_utils::find_qrcode_sign_enc_in_url(r[0].getText())
            } else {
                // 如果不是精确截取的二维码，则不需要提示。
                cxsign_utils::find_qrcode_sign_enc_in_url(url)
            };
        }
    }
    None
}
