use crate::{
    utils::download_image, IconClickImage, ObstacleImage, RotateImages, SlideImages, TextClickInfo,
};
use cxlib_error::{CaptchaError, InitError};
use image::DynamicImage;
use log::debug;
use onceinit::{OnceInit, UninitGlobal};
use ureq::Agent;
use yapt::point_2d::{Point, Point2D};

mod click_captcha_helper {
    use std::fmt::{Display, Formatter};
    use yapt::impl_point2d;

    #[derive(Debug)]
    pub struct Point<T>(T, T);
    impl_point2d!(Point<T>, t, t);
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
        assert_eq!(Point3(Point(61, 77), Point(128, 94), Point(210, 74)).to_string(),
                   "%5B%7B%22x%22%3A61%2C%22y%22%3A77%7D%2C%7B%22x%22%3A128%2C%22y%22%3A94%7D%2C%7B%22x%22%3A210%2C%22y%22%3A74%7D%5D")
    }
}
pub trait VerificationInfoTrait<I, O>:
    Sized + UninitGlobal<SolverRaw<I, O>, OnceInit<SolverRaw<I, O>>>
where
    I: 'static,
    O: 'static,
{
    /// 以自身的引用构造类型 `I`,
    /// 如：
    ///
    /// 验证信息可能包含图片 Url, 而计算验证结果需要图片类型，
    /// 则该函数需要做的应当为：下载图片并返回。
    fn prepare_data(self, agent: &Agent, referer: &str) -> Result<I, CaptchaError>;
    /// 默认过验证算法。如不实现则仅仅返回一个错误。
    fn default_solver(input: I) -> Result<O, CaptchaError> {
        let _ = input;
        Err(CaptchaError::UnsupportedType)
    }
    // /// 设置全局过验证算法所需。
    // ///
    // /// 若输入输出类型（`I`, `O`）与默认类型如 [`SlideImages`] 等一致，实现该函数时，你可以直接调用默认类型的实现。
    // ///
    // /// 若不一致，你需要自定义一个 static 的 [`OnceInit<SolverRaw<I, O>>`](OnceInit) 数据。
    // /// 另见 [`SolverRaw`].
    // fn static_solver_holder() -> &'static OnceInit<SolverRaw<I, O>>;
    /// 将结果转为字符串类型，用来向网站发送请求。
    fn result_to_string(result: O) -> String;
    fn solve(input: I) -> Result<O, CaptchaError>
    where
        Self: VerificationInfoTrait<I, O> + 'static,
        SolverRaw<I, O>: 'static,
    {
        match Self::holder().get() {
            Err(_) => Self::default_solver(input),
            Ok(s) => s(input),
        }
    }
    /// 初始化 `Solver`.
    ///
    /// 另见 [`VerificationInfoTrait::init_owned_solver`].
    fn init_solver(
        solver: &'static (impl Fn(I) -> Result<O, CaptchaError> + Sync),
    ) -> Result<(), InitError> {
        Ok(Self::init(solver)?)
    }
    /// 初始化 `Solver`.
    ///
    /// 另见 [`VerificationInfoTrait::init_solver`].
    fn init_owned_solver(
        solver: impl Fn(I) -> Result<O, CaptchaError> + Sync + 'static,
    ) -> Result<(), InitError> {
        let solver: Box<dyn Fn(_) -> _ + Sync + 'static> = Box::new(solver);
        Ok(Self::init_boxed(solver)?)
    }
    fn solver(self, agent: &Agent, referer: &str) -> Result<String, CaptchaError>
    where
        Self: 'static,
    {
        let data = self.prepare_data(agent, referer)?;
        let output = Self::solve(data)?;
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
/// 类型别名，本质上是一个 `dyn Fn` 类型。
pub type SolverRaw<I, O> = dyn Fn(I) -> Result<O, CaptchaError> + Sync;
type SlideSolverRaw = SolverRaw<(DynamicImage, DynamicImage), u32>;
type IconClickSolverRaw = SolverRaw<DynamicImage, TriplePoint<u32>>;
type TextClickSolverRaw = SolverRaw<(String, DynamicImage), TriplePoint<u32>>;
type RotateSolverRaw = SolverRaw<(DynamicImage, DynamicImage), u32>;
type ObstacleSolverRaw = SolverRaw<DynamicImage, Point<u32>>;
static SLIDE_SOLVER: OnceInit<SlideSolverRaw> = OnceInit::uninit();
static ICON_CLICK_SOLVER: OnceInit<IconClickSolverRaw> = OnceInit::uninit();
static TEXT_CLICK_SOLVER: OnceInit<TextClickSolverRaw> = OnceInit::uninit();
static ROTATE_SOLVER: OnceInit<RotateSolverRaw> = OnceInit::uninit();
static OBSTACLE_SOLVER: OnceInit<ObstacleSolverRaw> = OnceInit::uninit();
impl
    UninitGlobal<
        SolverRaw<(DynamicImage, DynamicImage), u32>,
        OnceInit<SolverRaw<(DynamicImage, DynamicImage), u32>>,
    > for SlideImages
{
    fn holder() -> &'static OnceInit<SolverRaw<(DynamicImage, DynamicImage), u32>> {
        &SLIDE_SOLVER
    }
}
impl VerificationInfoTrait<(DynamicImage, DynamicImage), u32> for SlideImages {
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
    #[cfg(not(feature = "slide_ui_solver"))]
    fn default_solver(
        (big_image, small_image): (DynamicImage, DynamicImage),
    ) -> Result<u32, CaptchaError> {
        use cxlib_imageproc::{
            find_sub_image,
            match_template::{match_template_for_slide, MatchTemplateMethod},
        };
        use cxlib_utils::time_it_and_print_result;
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
    fn result_to_string(result: u32) -> String {
        debug!("本地滑块结果：{result}");
        format!("%5B%7B%22x%22%3A{}%7D%5D", result)
    }
}
impl
    UninitGlobal<
        SolverRaw<DynamicImage, TriplePoint<u32>>,
        OnceInit<SolverRaw<DynamicImage, TriplePoint<u32>>>,
    > for IconClickImage
{
    fn holder() -> &'static OnceInit<SolverRaw<DynamicImage, TriplePoint<u32>>> {
        &ICON_CLICK_SOLVER
    }
}
impl VerificationInfoTrait<DynamicImage, TriplePoint<u32>> for IconClickImage {
    fn prepare_data(self, agent: &Agent, referer: &str) -> Result<DynamicImage, CaptchaError> {
        let img = download_image(agent, self.image_url(), referer)?;
        Ok(img)
    }
    #[cfg(feature = "icon_click_ui_solver")]
    fn default_solver(input: DynamicImage) -> Result<TriplePoint<u32>, CaptchaError> {
        use captcha_solver_ui::solvers::Marker;
        captcha_solver_ui::solvers::MIconClick::ui_solver(input).map_err(convert_captcha_error)
    }
    /// \[{"x":82,"y":114},{"x":286,"y":68},{"x":154,"y":90}\] <br/>
    /// x, y 为图标相对 origin_image 右上角的位置。
    fn result_to_string(result: TriplePoint<u32>) -> String {
        let points = click_captcha_helper::Point3::from((result.0, result.1, result.2));
        debug!("本地滑块结果：{points}");
        points.to_string()
    }
}

impl
    UninitGlobal<
        SolverRaw<(String, DynamicImage), TriplePoint<u32>>,
        OnceInit<SolverRaw<(String, DynamicImage), TriplePoint<u32>>>,
    > for TextClickInfo
{
    fn holder() -> &'static OnceInit<SolverRaw<(String, DynamicImage), TriplePoint<u32>>> {
        &TEXT_CLICK_SOLVER
    }
}
impl VerificationInfoTrait<(String, DynamicImage), TriplePoint<u32>> for TextClickInfo {
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
    #[cfg(feature = "text_click_ui_solver")]
    fn default_solver(input: (String, DynamicImage)) -> Result<TriplePoint<u32>, CaptchaError> {
        use captcha_solver_ui::solvers::Marker;
        captcha_solver_ui::solvers::MTextClick::ui_solver(input).map_err(convert_captcha_error)
    }
    /// \[{"x":82,"y":114},{"x":286,"y":68},{"x":154,"y":90}\] <br/>
    /// x, y 为图标相对 origin_image 右上角的位置。
    fn result_to_string(result: TriplePoint<u32>) -> String {
        IconClickImage::result_to_string(result)
    }
}
impl
    UninitGlobal<SolverRaw<DynamicImage, Point<u32>>, OnceInit<SolverRaw<DynamicImage, Point<u32>>>>
    for ObstacleImage
{
    fn holder() -> &'static OnceInit<SolverRaw<DynamicImage, Point<u32>>> {
        &OBSTACLE_SOLVER
    }
}
impl VerificationInfoTrait<DynamicImage, Point<u32>> for ObstacleImage {
    fn prepare_data(self, agent: &Agent, referer: &str) -> Result<DynamicImage, CaptchaError> {
        debug!("图片 url：{}", self.img_url());
        let img = download_image(agent, self.img_url(), referer)?;
        Ok(img)
    }
    #[cfg(feature = "obstacle_ui_solver")]
    fn default_solver(input: DynamicImage) -> Result<Point<u32>, CaptchaError> {
        use captcha_solver_ui::solvers::Marker;
        captcha_solver_ui::solvers::MObstacle::ui_solver(input).map_err(convert_captcha_error)
    }
    // TODO: 需要验证
    /// \[{"x":82,"y":114},{"x":286,"y":68},{"x":154,"y":90}\] <br/>
    /// x, y 为图标相对 origin_image 右上角的位置。
    fn result_to_string(result: Point<u32>) -> String {
        let data = click_captcha_helper::Point::from_point(result);
        debug!("本地滑块结果：{data}");
        format!("%5B{}%5D", data)
    }
}
impl
    UninitGlobal<
        SolverRaw<(DynamicImage, DynamicImage), u32>,
        OnceInit<SolverRaw<(DynamicImage, DynamicImage), u32>>,
    > for RotateImages
{
    fn holder() -> &'static OnceInit<SolverRaw<(DynamicImage, DynamicImage), u32>> {
        &ROTATE_SOLVER
    }
}
impl VerificationInfoTrait<(DynamicImage, DynamicImage), u32> for RotateImages {
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
    #[cfg(not(feature = "rotate_ui_solver"))]
    fn default_solver(input: (DynamicImage, DynamicImage)) -> Result<u32, CaptchaError> {
        use cxlib_imageproc::{
            match_template::{match_template_for_rotate, MatchTemplateMethod},
            rotate_captcha_utils::match_angle,
        };
        use cxlib_utils::time_it_and_print_result;
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
    /// result 取值为 0-280.
    ///
    /// 目前与旋转角度的换算关系为 angle/1.8.
    fn result_to_string(result: u32) -> String {
        debug!("本地旋转结果：{result}");
        format!("%5B%7B%22x%22%3A{}%7D%5D", result)
    }
}
