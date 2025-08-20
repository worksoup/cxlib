use std::marker::PhantomData;

use crate::{
    CaptchaError, SolveCaptcha, SolverProviderTrait, TriplePoint, VerificationInfoTrait,
    click_captcha_helper, utils::download_image,
};

use getset2::Getset2;
use image::DynamicImage;
use serde::Deserialize;
use ureq::Agent;

/// # Solver 签名
/// ```rust, no_run
///
///
/// type Solver = impl Fn(image::DynamicImage) -> Result<u32, cxlib_error::CaptchaError>;
/// ```
/// 调用 [`init_solver`](IconClickImage::init_solver) 或 [`init_owned_solver`](IconClickImage::init_owned_solver) 完成初始化。
#[derive(Debug, Deserialize, Getset2)]
#[getset2(get_ref(pub))]
pub struct IconClickImage<SolverProvider> {
    // #[serde(rename = "type")]
    // _captcha_type: String,
    #[serde(rename = "originImage")]
    image_url: String,
    #[serde(skip)]
    marker: PhantomData<SolverProvider>,
}

impl<SolverProvider> VerificationInfoTrait for IconClickImage<SolverProvider> {
    type I = DynamicImage;
    type O = TriplePoint<u32>;

    #[inline]
    fn captcha_type() -> &'static str {
        "iconclick"
    }
    #[inline]
    fn prepare_data(self, agent: &Agent, referer: &str) -> Result<DynamicImage, CaptchaError> {
        let img = download_image(agent, self.image_url(), referer)?;
        Ok(img)
    }
    /// \[{"x":82,"y":114},{"x":286,"y":68},{"x":154,"y":90}\] <br/>
    /// x, y 为图标相对 origin_image 右上角的位置。
    #[inline]
    fn result_to_string(result: TriplePoint<u32>) -> String {
        click_captcha_helper::triple_point_to_string(result)
    }
}
impl<SolverProvider> SolveCaptcha for IconClickImage<SolverProvider>
where
    SolverProvider: SolverProviderTrait<I = Self::I, O = Self::O>,
{
    type SolverProvider = SolverProvider;
}
