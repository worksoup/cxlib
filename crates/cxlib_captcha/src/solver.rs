use cxlib_error::Error;
use cxlib_imageproc::{find_max_ncc, find_sub_image, Point};
use image::DynamicImage;
use log::debug;
use onceinit::{OnceInit, OnceInitError};
use serde::{de::DeserializeOwned, Deserialize};
use ureq::{serde_json, Agent};

mod click_captcha_helper {
    use std::fmt::{Display, Formatter};

    #[derive(Debug)]
    pub struct Point<T>(T, T);
    impl<T: Display> Display for Point<T> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "%7B%22x%22%3A{}%2C%22y%22%3A{}%7D", self.0, self.1)
        }
    }
    impl<T> From<cxlib_imageproc::Point<T>> for Point<T> {
        fn from(value: cxlib_imageproc::Point<T>) -> Self {
            Self(value.x, value.y)
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
            cxlib_imageproc::Point<T>,
            cxlib_imageproc::Point<T>,
            cxlib_imageproc::Point<T>,
        )> for Point3<T>
    {
        fn from(
            value: (
                cxlib_imageproc::Point<T>,
                cxlib_imageproc::Point<T>,
                cxlib_imageproc::Point<T>,
            ),
        ) -> Self {
            Self(value.0.into(), value.1.into(), value.2.into())
        }
    }
    #[cfg(test)]
    #[test]
    fn test() {
        assert_eq!(Point3(Point(61, 77), Point(128, 94), Point(210, 74)).to_string(),
                   "%5B%7B%22x%22%3A61%2C%22y%22%3A77%7D%2C%7B%22x%22%3A128%2C%22y%22%3A94%7D%2C%7B%22x%22%3A210%2C%22y%22%3A74%7D%5D")
    }
}
pub trait VerificationInfoTrait<I, O> {
    fn prepare_data(
        &self,
        agent: &Agent,
        referer: &str,
    ) -> std::result::Result<I, cxlib_error::Error>;
    fn default_solver(input: I) -> Result<O> {
        let _ = input;
        Err(Error::CaptchaError("不支持该类型的验证码。".to_owned()))
    }
    fn static_solver_holder() -> &'static OnceInit<SolverRaw<I, O>>;
    fn result_to_string(result: &O) -> String;
    fn solve(input: I) -> Result<O>
    where
        Self: VerificationInfoTrait<I, O> + 'static,
        SolverRaw<I, O>: 'static,
    {
        match Self::static_solver_holder().get_data() {
            Ok(s) => s(input),
            Err(_) => Self::default_solver(input),
        }
    }
    /// 初始化 `Solver`.
    ///
    /// 另见 [`VerificationInfoTrait::init_owned_solver`].
    fn init_solver<F: Fn(I) -> Result<O> + Sync>(
        solver: &'static F,
    ) -> std::result::Result<(), OnceInitError>
    where
        I: 'static,
        O: 'static,
    {
        Self::static_solver_holder().set_data(solver)
    }
    /// 初始化 `Solver`.
    ///
    /// 另见 [`VerificationInfoTrait::init_solver`].
    fn init_owned_solver<F: Fn(I) -> Result<O> + Sync + 'static>(
        solver: F,
    ) -> std::result::Result<(), OnceInitError>
    where
        I: 'static,
        O: 'static,
        Self: VerificationInfoTrait<I, O>,
    {
        let solver: Box<dyn Fn(_) -> _ + Sync + 'static> = Box::new(solver);
        Self::static_solver_holder().set_boxed_data(solver)
    }
    fn solver(agent: &Agent, image: serde_json::Value, referer: &str) -> Result<String>
    where
        Self: DeserializeOwned + 'static,
        SolverRaw<I, O>: 'static,
    {
        let self_: Self = serde_json::from_value(image).unwrap();
        self_.solver_inner(agent, referer)
    }
    fn solver_inner(&self, agent: &Agent, referer: &str) -> Result<String>
    where
        Self: 'static,
        SolverRaw<I, O>: 'static,
    {
        let data = self.prepare_data(agent, referer)?;
        let output = Self::solve(data)?;
        let r = Self::result_to_string(&output);
        Ok(r)
    }
}
type Result<T> = std::result::Result<T, cxlib_error::Error>;
type TriplePoint<T> = (Point<T>, Point<T>, Point<T>);
type SolverRaw<I, O> = dyn Fn(I) -> Result<O> + Sync;
type SlideSolverRaw = SolverRaw<(DynamicImage, DynamicImage), u32>;
type IconClickSolverRaw = SolverRaw<DynamicImage, TriplePoint<u32>>;
type TextClickSolverRaw = SolverRaw<(String, DynamicImage), TriplePoint<u32>>;
type RotateSolverRaw = SolverRaw<(DynamicImage, DynamicImage), u32>;
type ObstacleSolverRaw = SolverRaw<DynamicImage, Point<u32>>;
static SLIDE_SOLVER: OnceInit<SlideSolverRaw> = OnceInit::new();
static ICON_CLICK_SOLVER: OnceInit<IconClickSolverRaw> = OnceInit::new();
static TEXT_CLICK_SOLVER: OnceInit<TextClickSolverRaw> = OnceInit::new();
static ROTATE_SOLVER: OnceInit<RotateSolverRaw> = OnceInit::new();
static OBSTACLE_SOLVER: OnceInit<ObstacleSolverRaw> = OnceInit::new();
/// Solver 签名：
/// ```rust, no_run
/// type Solver = impl Fn((
///     image::DynamicImage, // big_img
///     image::DynamicImage, // small_img
/// )) -> Result<u32, cxlib_error::Error>;
/// ```
/// 调用 [`init_solver`](SlideImages::init_solver) 或 [`init_owned_solver`](SlideImages::init_owned_solver) 完成初始化。
#[derive(Debug, Deserialize)]
pub struct SlideImages {
    #[serde(rename = "shadeImage")]
    big_img_url: String,
    #[serde(rename = "cutoutImage")]
    small_img_url: String,
}

impl VerificationInfoTrait<(DynamicImage, DynamicImage), u32> for SlideImages {
    fn prepare_data(
        &self,
        agent: &Agent,
        referer: &str,
    ) -> std::result::Result<(DynamicImage, DynamicImage), cxlib_error::Error> {
        debug!("small_image_url：{}", self.small_img_url);
        debug!("big_image_url：{}", self.big_img_url);
        let small_img = cxlib_imageproc::download_image(agent, &self.small_img_url, referer)?;
        let big_img = cxlib_imageproc::download_image(agent, &self.big_img_url, referer)?;
        Ok((big_img, small_img))
    }
    fn default_solver(input: (DynamicImage, DynamicImage)) -> std::result::Result<u32, Error> {
        Ok(find_sub_image(&input.0, &input.1, find_max_ncc))
    }
    fn static_solver_holder() -> &'static OnceInit<SlideSolverRaw> {
        &SLIDE_SOLVER
    }
    fn result_to_string(result: &u32) -> String {
        debug!("本地滑块结果：{result}");
        format!("%5B%7B%22x%22%3A{}%7D%5D", result)
    }
}
/// Solver 签名：
/// ```rust, no_run
/// type Solver = impl Fn(image::DynamicImage) -> Result<u32, cxlib_error::Error>;
/// ```
/// 调用 [`init_solver`](IconClickImage::init_solver) 或 [`init_owned_solver`](IconClickImage::init_owned_solver) 完成初始化。
#[derive(Debug, Deserialize)]
pub struct IconClickImage {
    // #[serde(rename = "type")]
    // _captcha_type: String,
    #[serde(rename = "originImage")]
    image_url: String,
}
impl VerificationInfoTrait<DynamicImage, TriplePoint<u32>> for IconClickImage {
    fn prepare_data(
        &self,
        agent: &Agent,
        referer: &str,
    ) -> std::result::Result<DynamicImage, cxlib_error::Error> {
        let img = cxlib_imageproc::download_image(agent, &self.image_url, referer)?;
        Ok(img)
    }
    fn static_solver_holder() -> &'static OnceInit<IconClickSolverRaw> {
        &ICON_CLICK_SOLVER
    }
    fn result_to_string(result: &TriplePoint<u32>) -> String {
        let points = click_captcha_helper::Point3::from((result.0, result.1, result.2));
        debug!("本地滑块结果：{points}");
        points.to_string()
    }
}

/// Solver 签名：
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
#[derive(Deserialize)]
pub struct TextClickInfo {
    // #[serde(rename = "type")]
    // _captcha_type: String,
    #[serde(rename = "context")]
    hanzi: String,
    #[serde(rename = "originImage")]
    img_url: String,
}
impl VerificationInfoTrait<(String, DynamicImage), TriplePoint<u32>> for TextClickInfo {
    fn prepare_data(
        &self,
        agent: &Agent,
        referer: &str,
    ) -> std::result::Result<(String, DynamicImage), cxlib_error::Error> {
        debug!("点选文字：{}", self.hanzi);
        debug!("图片 url：{}", self.img_url);
        let img = cxlib_imageproc::download_image(agent, &self.img_url, referer)?;
        Ok((self.hanzi.clone(), img))
    }
    fn static_solver_holder() -> &'static OnceInit<TextClickSolverRaw> {
        &TEXT_CLICK_SOLVER
    }
    fn result_to_string(result: &TriplePoint<u32>) -> String {
        IconClickImage::result_to_string(result)
    }
}
/// Solver 签名：
/// ```rust, no_run
/// type Solver = impl Fn(image::DynamicImage) -> Result<cxlib_imageproc::Point<u32>, cxlib_error::Error>;
/// ```
/// 调用 [`init_solver`](ObstacleImage::init_solver) 或 [`init_owned_solver`](ObstacleImage::init_owned_solver) 完成初始化。
#[derive(Deserialize)]
pub struct ObstacleImage {
    // #[serde(rename = "type")]
    // _captcha_type: String,
    #[serde(rename = "originImage")]
    img_url: String,
}
impl VerificationInfoTrait<DynamicImage, Point<u32>> for ObstacleImage {
    fn prepare_data(
        &self,
        agent: &Agent,
        referer: &str,
    ) -> std::result::Result<DynamicImage, cxlib_error::Error> {
        debug!("图片 url：{}", self.img_url);
        let img = cxlib_imageproc::download_image(agent, &self.img_url, referer)?;
        Ok(img)
    }
    fn static_solver_holder() -> &'static OnceInit<ObstacleSolverRaw> {
        &OBSTACLE_SOLVER
    }
    fn result_to_string(result: &Point<u32>) -> String {
        let data = click_captcha_helper::Point::from(*result);
        debug!("本地滑块结果：{data}");
        format!("%5B{}%5D", data)
    }
}
/// Solver 签名：
/// ```rust, no_run
/// type Solver = impl Fn((
///     image::DynamicImage, // fixed_img
///     image::DynamicImage, // rotatable_img
/// )) -> Result<u32, cxlib_error::Error>;
/// ```
/// 调用 [`init_solver`](RotateImages::init_solver) 或 [`init_owned_solver`](RotateImages::init_owned_solver) 完成初始化。
#[derive(Deserialize)]
pub struct RotateImages {
    #[serde(rename = "shadeImage")]
    fixed_img_url: String,
    #[serde(rename = "cutoutImage")]
    rotatable_img_url: String,
}
impl VerificationInfoTrait<(DynamicImage, DynamicImage), u32> for RotateImages {
    fn prepare_data(
        &self,
        agent: &Agent,
        referer: &str,
    ) -> std::result::Result<(DynamicImage, DynamicImage), cxlib_error::Error> {
        debug!(
            "验证码图片 url：{}, {}",
            self.fixed_img_url, self.rotatable_img_url
        );
        let rotatable_img =
            cxlib_imageproc::download_image(agent, &self.rotatable_img_url, referer)?;
        let fixed_img = cxlib_imageproc::download_image(agent, &self.fixed_img_url, referer)?;
        Ok((fixed_img, rotatable_img))
    }
    fn static_solver_holder() -> &'static OnceInit<RotateSolverRaw> {
        &ROTATE_SOLVER
    }
    /// \[{"x":82,"y":114},{"x":286,"y":68},{"x":154,"y":90}\] <br/>
    /// x, y 为图标相对 origin_image 右上角的位置。
    fn result_to_string(result: &u32) -> String {
        debug!("本地旋转结果：{result}");
        format!("%5B%7B%22x%22%3A{}%7D%5D", result)
    }
}
