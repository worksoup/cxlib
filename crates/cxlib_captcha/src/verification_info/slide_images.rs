use cxlib_imageproc::Point;
use getset2::Getset2;
use image::DynamicImage;
use log::debug;
use serde::Deserialize;
use std::marker::PhantomData;
use ureq::Agent;

use crate::{CaptchaError, SolverProviderTrait, VerificationInfoTrait, utils::download_image};
pub struct DefaultSlideImagesSolverProvider;
impl SolverProviderTrait for DefaultSlideImagesSolverProvider {
    type I = (image::DynamicImage, image::DynamicImage);

    type O = u32;

    #[inline]
    fn solver((big_image, small_image): Self::I) -> Result<Self::O, crate::CaptchaError> {
        use cx_debug_utils::time_it_and_print_result;
        use cxlib_imageproc::{
            find_sub_image,
            match_template::{MatchTemplateMethod, match_template_for_slide},
        };
        time_it_and_print_result(move || {
            Ok(find_sub_image(&big_image, &small_image, |a, b, mask| {
                match_template_for_slide(a, b, MatchTemplateMethod::SumOfSquaredErrors, mask)
            }))
        })
    }
}
/// # Solver 签名
/// ```rust, no_run
/// type Solver = impl Fn((
///     image::DynamicImage, // big_img
///     image::DynamicImage, // small_img
/// )) -> Result<u32, cxlib_error::CaptchaError>;
/// ```
/// 调用 [`init_solver`](SlideImages::init_solver) 或 [`init_owned_solver`](SlideImages::init_owned_solver) 完成初始化。
#[derive(Debug, Deserialize, Getset2)]
#[getset2(get_ref(pub))]
pub struct SlideImages<SolverProvider = DefaultSlideImagesSolverProvider> {
    #[serde(rename = "shadeImage")]
    big_img_url: String,
    #[serde(rename = "cutoutImage")]
    small_img_url: String,
    #[serde(skip)]
    marker: PhantomData<SolverProvider>,
}
/// 类型别名，三个一组的 [`Point`] 类型。
pub type TriplePoint<T> = (Point<T>, Point<T>, Point<T>);
impl<SolverProvider: SolverProviderTrait<I = (image::DynamicImage, image::DynamicImage), O = u32>>
    VerificationInfoTrait for SlideImages<SolverProvider>
{
    type I = SolverProvider::I;
    type O = SolverProvider::O;

    #[inline]
    fn captcha_type() -> &'static str {
        "slide"
    }
    #[inline]
    fn prepare_data(
        self,
        agent: &Agent,
        referer: &str,
    ) -> Result<(DynamicImage, DynamicImage), CaptchaError> {
        debug!("small_image_url：{}", self.small_img_url());
        debug!("big_image_url：{}", self.big_img_url());
        let small_img = download_image(agent, self.small_img_url(), referer)?;
        let big_img = download_image(agent, self.big_img_url(), referer)?;
        Ok((big_img, small_img))
    }
    #[inline]
    fn result_to_string(result: u32) -> String {
        debug!("本地滑块结果：{result}");
        format!("%5B%7B%22x%22%3A{result}%7D%5D",)
    }
    #[inline]
    fn default_solver(input: (DynamicImage, DynamicImage)) -> Result<u32, CaptchaError> {
        SolverProvider::solver(input)
    }
}
