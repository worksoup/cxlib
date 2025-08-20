use std::marker::PhantomData;

use crate::{
    CaptchaError, DefaultSolver, TriplePoint, VerificationInfoTrait, click_captcha_helper,
    utils::download_image,
};
use getset2::Getset2;
use image::DynamicImage;
use log::debug;
use serde::Deserialize;
use ureq::Agent;

/// # Solver 签名
/// ```rust, no_run
/// type Solver = impl Fn((
///     String,              // hanzi
///     image::DynamicImage,
/// )) -> Result<(
///     cxlib_imageproc::Point<u32>, // 1
///     cxlib_imageproc::Point<u32>, // 2
///     cxlib_imageproc::Point<u32>, // 3
/// ), cxlib_error::CaptchaError>;
/// ```
/// 调用 [`init_solver`](TextClickInfo::init_solver) 或 [`init_owned_solver`](TextClickInfo::init_owned_solver) 完成初始化。
#[derive(Debug, Deserialize, Getset2)]
#[getset2(get_ref(pub))]
pub struct TextClickInfo<SolverProvider> {
    // #[serde(rename = "type")]
    // _captcha_type: String,
    #[serde(rename = "context")]
    hanzi: String,
    #[serde(rename = "originImage")]
    img_url: String,
    #[serde(skip)]
    marker: PhantomData<SolverProvider>,
}
impl<SolverProvider> VerificationInfoTrait for TextClickInfo<SolverProvider> {
    type I = (String, DynamicImage);
    type O = TriplePoint<u32>;

    #[inline]
    fn captcha_type() -> &'static str {
        "textclick"
    }
    #[inline]
    fn prepare_data(
        self,
        agent: &Agent,
        referer: &str,
    ) -> Result<(String, DynamicImage), CaptchaError> {
        debug!("点选文字：{}", self.hanzi());
        debug!("图片 url：{}", self.img_url());
        let img = download_image(agent, self.img_url(), referer)?;
        Ok((self.hanzi().clone(), img))
    }
    /// \[{"x":82,"y":114},{"x":286,"y":68},{"x":154,"y":90}\] <br/>
    /// x, y 为图标相对 origin_image 右上角的位置。
    #[inline]
    fn result_to_string(result: TriplePoint<u32>) -> String {
        click_captcha_helper::triple_point_to_string(result)
    }
}
impl<SolverProvider> DefaultSolver<SolverProvider> for TextClickInfo<SolverProvider> {}
