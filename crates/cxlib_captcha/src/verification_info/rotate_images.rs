use getset2::Getset2;
use image::DynamicImage;
use log::debug;
use serde::Deserialize;
use std::marker::PhantomData;
use ureq::Agent;

use crate::{CaptchaError, SolverProviderTrait, VerificationInfoTrait, utils::download_image};
pub struct DefaultRotateImagesSolverProvider;
impl SolverProviderTrait for DefaultRotateImagesSolverProvider {
    type I = (image::DynamicImage, image::DynamicImage);

    type O = u32;

    #[inline]
    fn solver(input: Self::I) -> Result<Self::O, crate::CaptchaError> {
        use cx_debug_utils::time_it_and_print_result;
        use cxlib_imageproc::{
            match_template::{MatchTemplateMethod, match_template_for_rotate},
            rotate_captcha_utils::match_angle,
        };
        time_it_and_print_result(move || {
            Ok(match_angle(&input.0, &input.1, |a, b| {
                match_template_for_rotate(a, b, MatchTemplateMethod::SumOfSquaredErrors)
            }))
        })
    }
}
/// # Solver 签名
/// ```rust, no_run
/// type Solver = impl Fn((
///     image::DynamicImage, // fixed_img
///     image::DynamicImage, // rotatable_img
/// )) -> Result<u32, cxlib_error::CaptchaError>;
/// ```
/// 调用 [`init_solver`](RotateImages::init_solver) 或 [`init_owned_solver`](RotateImages::init_owned_solver) 完成初始化。
#[derive(Debug, Deserialize, Getset2)]
#[getset2(get_ref(pub))]
pub struct RotateImages<SolverProvider = DefaultRotateImagesSolverProvider> {
    #[serde(rename = "shadeImage")]
    rotatable_img_url: String,
    #[serde(rename = "cutoutImage")]
    fixed_img_url: String,
    #[serde(skip)]
    marker: PhantomData<SolverProvider>,
}

impl<SolverProvider: SolverProviderTrait<I = (image::DynamicImage, image::DynamicImage), O = u32>>
    VerificationInfoTrait for RotateImages<SolverProvider>
{
    type I = SolverProvider::I;
    type O = SolverProvider::O;

    #[inline]
    fn captcha_type() -> &'static str {
        "rotate"
    }
    #[inline]
    fn prepare_data(
        self,
        agent: &Agent,
        referer: &str,
    ) -> Result<(DynamicImage, DynamicImage), CaptchaError> {
        debug!(
            "验证码图片 url：{}, {}",
            self.fixed_img_url(),
            self.rotatable_img_url()
        );
        let fixed_img = download_image(agent, self.fixed_img_url(), referer)?;
        let rotatable_img = download_image(agent, self.rotatable_img_url(), referer)?;
        Ok((fixed_img, rotatable_img))
    }
    /// result 取值为 0-280.
    ///
    /// 目前与旋转角度的换算关系为 angle/1.8.
    #[inline]
    fn result_to_string(result: u32) -> String {
        debug!("本地旋转结果：{result}");
        format!("%5B%7B%22x%22%3A{result}%7D%5D")
    }
    #[inline]
    fn default_solver(input: (DynamicImage, DynamicImage)) -> Result<u32, CaptchaError> {
        SolverProvider::solver(input)
    }
}
