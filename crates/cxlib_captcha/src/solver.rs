use crate::{
    CaptchaError, IconClickImage, ObstacleImage, RotateImages, SlideImages, TextClickInfo,
    utils::download_image,
};
use image::DynamicImage;
use log::debug;
use ureq::Agent;
use yapt::point_2d::{Point, Point2D};

mod click_captcha_helper {
    use std::fmt::{Display, Formatter};
    use yapt::impl_point2d;

    #[derive(Debug)]
    pub struct Point<T>(T, T);
    impl_point2d!(Point<T>, Tuple, Tuple);
    impl<T: Display> Display for Point<T> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "%7B%22x%22%3A{}%2C%22y%22%3A{}%7D", self.0, self.1)
        }
    }
    #[derive(Debug)]
    pub struct Point3<T>(Point<T>, Point<T>, Point<T>);
    impl<T: Display> Display for Point3<T> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "%5B{}%2C{}%2C{}%5D", self.0, self.1, self.2)
        }
    }
    impl<T> From<(Point<T>, Point<T>, Point<T>)> for Point3<T> {
        fn from(value: (Point<T>, Point<T>, Point<T>)) -> Self {
            Self(value.0, value.1, value.2)
        }
    }
    impl<T>
        From<(
            yapt::point_2d::Point<T>,
            yapt::point_2d::Point<T>,
            yapt::point_2d::Point<T>,
        )> for Point3<T>
    {
        fn from(
            value: (
                yapt::point_2d::Point<T>,
                yapt::point_2d::Point<T>,
                yapt::point_2d::Point<T>,
            ),
        ) -> Self {
            Self(
                value.0.into_point_2d(),
                value.1.into_point_2d(),
                value.2.into_point_2d(),
            )
        }
    }
    #[cfg(test)]
    #[test]
    fn test() {
        assert_eq!(
            Point3(Point(61, 77), Point(128, 94), Point(210, 74)).to_string(),
            "%5B%7B%22x%22%3A61%2C%22y%22%3A77%7D%2C%7B%22x%22%3A128%2C%22y%22%3A94%7D%2C%7B%22x%22%3A210%2C%22y%22%3A74%7D%5D"
        )
    }
}
pub trait VerificationInfoTrait: Sized {
    type I: 'static;
    type O: 'static;
    fn captcha_type() -> &'static str;
    /// 以自身的引用构造类型 `I`,
    /// 如：
    ///
    /// 验证信息可能包含图片 Url, 而计算验证结果需要图片类型，
    /// 则该函数需要做的应当为：下载图片并返回。
    fn prepare_data(self, agent: &Agent, referer: &str) -> Result<Self::I, CaptchaError>;
    /// 将结果转为字符串类型，用来向网站发送请求。
    fn result_to_string(result: Self::O) -> String;
    /// 过验证算法。如不实现则仅仅返回一个错误。
    fn default_solver(input: Self::I) -> Result<Self::O, CaptchaError> {
        let _ = input;
        Err(CaptchaError::UnsupportedType)
    }
    fn solve(self, agent: &Agent, referer: &str) -> Result<String, CaptchaError> {
        let data = self.prepare_data(agent, referer)?;
        let output = Self::default_solver(data)?;
        let r = Self::result_to_string(output);
        Ok(r)
    }
}
#[cfg(feature = "ui_solver")]
#[allow(dead_code)]
fn convert_captcha_error(a: captcha_solver_ui::CaptchaError) -> CaptchaError {
    match a {
        captcha_solver_ui::CaptchaError::VerifyFailed => CaptchaError::VerifyFailed,
        captcha_solver_ui::CaptchaError::Canceled(s) => CaptchaError::Canceled(s),
    }
}
/// 类型别名，三个一组的 [`Point`] 类型。
pub type TriplePoint<T> = (Point<T>, Point<T>, Point<T>);
impl VerificationInfoTrait for SlideImages {
    type I = (DynamicImage, DynamicImage);
    type O = u32;

    fn captcha_type() -> &'static str {
        "slide"
    }
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
    fn result_to_string(result: u32) -> String {
        debug!("本地滑块结果：{result}");
        format!("%5B%7B%22x%22%3A{result}%7D%5D",)
    }
    #[cfg(not(feature = "slide_ui_solver"))]
    fn default_solver(
        (big_image, small_image): (DynamicImage, DynamicImage),
    ) -> Result<u32, CaptchaError> {
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

    #[cfg(feature = "slide_ui_solver")]
    fn default_solver(input: (DynamicImage, DynamicImage)) -> Result<u32, CaptchaError> {
        use captcha_solver_ui::solvers::Marker;
        captcha_solver_ui::solvers::MSlide::ui_solver(input).map_err(convert_captcha_error)
    }
}
impl VerificationInfoTrait for IconClickImage {
    type I = DynamicImage;
    type O = TriplePoint<u32>;

    fn captcha_type() -> &'static str {
        "iconclick"
    }
    fn prepare_data(self, agent: &Agent, referer: &str) -> Result<DynamicImage, CaptchaError> {
        let img = download_image(agent, self.image_url(), referer)?;
        Ok(img)
    }
    /// \[{"x":82,"y":114},{"x":286,"y":68},{"x":154,"y":90}\] <br/>
    /// x, y 为图标相对 origin_image 右上角的位置。
    fn result_to_string(result: TriplePoint<u32>) -> String {
        let points = click_captcha_helper::Point3::from((result.0, result.1, result.2));
        debug!("本地滑块结果：{points}");
        points.to_string()
    }

    #[cfg(feature = "icon_click_ui_solver")]
    fn default_solver(input: DynamicImage) -> Result<TriplePoint<u32>, CaptchaError> {
        use captcha_solver_ui::solvers::Marker;
        captcha_solver_ui::solvers::MIconClick::ui_solver(input).map_err(convert_captcha_error)
    }
}
impl VerificationInfoTrait for TextClickInfo {
    type I = (String, DynamicImage);
    type O = TriplePoint<u32>;

    fn captcha_type() -> &'static str {
        "textclick"
    }
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
    fn result_to_string(result: TriplePoint<u32>) -> String {
        IconClickImage::result_to_string(result)
    }

    #[cfg(feature = "text_click_ui_solver")]
    fn default_solver(input: (String, DynamicImage)) -> Result<TriplePoint<u32>, CaptchaError> {
        use captcha_solver_ui::solvers::Marker;
        captcha_solver_ui::solvers::MTextClick::ui_solver(input).map_err(convert_captcha_error)
    }
}

impl VerificationInfoTrait for ObstacleImage {
    type I = DynamicImage;
    type O = Point<u32>;

    fn captcha_type() -> &'static str {
        "obstacle"
    }
    fn prepare_data(self, agent: &Agent, referer: &str) -> Result<DynamicImage, CaptchaError> {
        debug!("图片 url：{}", self.img_url());
        let img = download_image(agent, self.img_url(), referer)?;
        Ok(img)
    }
    // TODO: 需要验证
    /// \[{"x":82,"y":114},{"x":286,"y":68},{"x":154,"y":90}\] <br/>
    /// x, y 为图标相对 origin_image 右上角的位置。
    fn result_to_string(result: Point<u32>) -> String {
        let data = click_captcha_helper::Point::from_point(result);
        debug!("本地滑块结果：{data}");
        format!("%5B{data}%5D",)
    }

    #[cfg(feature = "obstacle_ui_solver")]
    fn default_solver(input: DynamicImage) -> Result<Point<u32>, CaptchaError> {
        use captcha_solver_ui::solvers::Marker;
        captcha_solver_ui::solvers::MObstacle::ui_solver(input).map_err(convert_captcha_error)
    }
}
impl VerificationInfoTrait for RotateImages {
    type I = (DynamicImage, DynamicImage);
    type O = u32;
    fn captcha_type() -> &'static str {
        "rotate"
    }
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
    fn result_to_string(result: u32) -> String {
        debug!("本地旋转结果：{result}");
        format!("%5B%7B%22x%22%3A{result}%7D%5D")
    }
    #[cfg(not(feature = "rotate_ui_solver"))]
    fn default_solver(input: (DynamicImage, DynamicImage)) -> Result<u32, CaptchaError> {
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

    #[cfg(feature = "rotate_ui_solver")]
    fn default_solver(input: (DynamicImage, DynamicImage)) -> Result<u32, CaptchaError> {
        use captcha_solver_ui::solvers::Marker;
        captcha_solver_ui::solvers::MRotate::ui_solver(input).map_err(convert_captcha_error)
    }
}
