use std::marker::PhantomData;

use crate::{CaptchaError, SolverProviderTrait, VerificationInfoTrait, utils::download_image};
use getset2::Getset2;
use image::DynamicImage;
use log::debug;
use serde::Deserialize;
use ureq::Agent;
use yapt::point_2d::{Point, Point2D};

use crate::click_captcha_helper;
/// # Solver 签名
/// ```rust, no_run
/// type Solver = impl Fn(image::DynamicImage) -> Result<cxlib_imageproc::Point<u32>, cxlib_error::CaptchaError>;
/// ```
/// 调用 [`init_solver`](ObstacleImage::init_solver) 或 [`init_owned_solver`](ObstacleImage::init_owned_solver) 完成初始化。
#[derive(Debug, Deserialize, Getset2)]
#[getset2(get_ref(pub))]
pub struct ObstacleImage<SolverProvider> {
    // #[serde(rename = "type")]
    // _captcha_type: String,
    #[serde(rename = "originImage")]
    img_url: String,
    #[serde(skip)]
    marker: PhantomData<SolverProvider>,
}

impl<SolverProvider: SolverProviderTrait<I = DynamicImage, O = Point<u32>>> VerificationInfoTrait
    for ObstacleImage<SolverProvider>
{
    type I = SolverProvider::I;
    type O = SolverProvider::O;

    #[inline]
    fn captcha_type() -> &'static str {
        "obstacle"
    }
    #[inline]
    fn prepare_data(self, agent: &Agent, referer: &str) -> Result<DynamicImage, CaptchaError> {
        debug!("图片 url：{}", self.img_url());
        let img = download_image(agent, self.img_url(), referer)?;
        Ok(img)
    }
    // TODO: 需要验证
    /// \[{"x":82,"y":114},{"x":286,"y":68},{"x":154,"y":90}\] <br/>
    /// x, y 为图标相对 origin_image 右上角的位置。
    #[inline]
    fn result_to_string(result: Point<u32>) -> String {
        let data = click_captcha_helper::Point::from_point(result);
        debug!("本地滑块结果：{data}");
        format!("%5B{data}%5D",)
    }

    #[inline]
    fn default_solver(input: DynamicImage) -> Result<Point<u32>, CaptchaError> {
        SolverProvider::solver(input)
    }
}
