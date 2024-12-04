use getset2::Getters;
use serde::Deserialize;

/// # Solver 签名
/// ```rust, no_run
/// type Solver = impl Fn((
///     image::DynamicImage, // big_img
///     image::DynamicImage, // small_img
/// )) -> Result<u32, cxlib_error::Error>;
/// ```
/// 调用 [`init_solver`](SlideImages::init_solver) 或 [`init_owned_solver`](SlideImages::init_owned_solver) 完成初始化。
#[derive(Debug, Deserialize, Getters)]
#[getset(get = "pub")]
pub struct SlideImages {
    #[serde(rename = "shadeImage")]
    big_img_url: String,
    #[serde(rename = "cutoutImage")]
    small_img_url: String,
}
/// # Solver 签名
/// ```rust, no_run
/// type Solver = impl Fn(image::DynamicImage) -> Result<u32, cxlib_error::Error>;
/// ```
/// 调用 [`init_solver`](IconClickImage::init_solver) 或 [`init_owned_solver`](IconClickImage::init_owned_solver) 完成初始化。
#[derive(Debug, Deserialize, Getters)]
#[getset(get = "pub")]
pub struct IconClickImage {
    // #[serde(rename = "type")]
    // _captcha_type: String,
    #[serde(rename = "originImage")]
    image_url: String,
}
/// # Solver 签名
/// ```rust, no_run
/// type Solver = impl Fn((
///     String,              // hanzi
///     image::DynamicImage,
/// )) -> Result<(
///     cxlib_imageproc::Point<u32>, // 1
///     cxlib_imageproc::Point<u32>, // 2
///     cxlib_imageproc::Point<u32>, // 3
/// ), cxlib_error::Error>;
/// ```
/// 调用 [`init_solver`](TextClickInfo::init_solver) 或 [`init_owned_solver`](TextClickInfo::init_owned_solver) 完成初始化。
#[derive(Debug, Deserialize, Getters)]
#[getset(get = "pub")]
pub struct TextClickInfo {
    // #[serde(rename = "type")]
    // _captcha_type: String,
    #[serde(rename = "context")]
    hanzi: String,
    #[serde(rename = "originImage")]
    img_url: String,
}
/// # Solver 签名
/// ```rust, no_run
/// type Solver = impl Fn(image::DynamicImage) -> Result<cxlib_imageproc::Point<u32>, cxlib_error::Error>;
/// ```
/// 调用 [`init_solver`](ObstacleImage::init_solver) 或 [`init_owned_solver`](ObstacleImage::init_owned_solver) 完成初始化。
#[derive(Debug, Deserialize, Getters)]
#[getset(get = "pub")]
pub struct ObstacleImage {
    // #[serde(rename = "type")]
    // _captcha_type: String,
    #[serde(rename = "originImage")]
    img_url: String,
}
/// # Solver 签名
/// ```rust, no_run
/// type Solver = impl Fn((
///     image::DynamicImage, // fixed_img
///     image::DynamicImage, // rotatable_img
/// )) -> Result<u32, cxlib_error::Error>;
/// ```
/// 调用 [`init_solver`](RotateImages::init_solver) 或 [`init_owned_solver`](RotateImages::init_owned_solver) 完成初始化。
#[derive(Debug, Deserialize, Getters)]
#[getset(get = "pub")]
pub struct RotateImages {
    #[serde(rename = "shadeImage")]
    fixed_img_url: String,
    #[serde(rename = "cutoutImage")]
    rotatable_img_url: String,
}
