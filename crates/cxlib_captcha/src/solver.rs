use crate::{CaptchaType, IconClickImage, ObstacleImage, RotateImages, SlideImages, TextClickInfo};
use cxlib_error::CaptchaError;
use cxlib_imageproc::{find_sub_image, image_from_bytes, Point};
use cxlib_utils::{time_it_and_print_result, ureq_get_bytes};
use image::DynamicImage;
use log::debug;
use onceinit::{OnceInit, OnceInitError};
use serde::de::DeserializeOwned;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
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
pub struct TopSolver;
type TopSolverGlobalInner = fn(&Agent, serde_json::Value, &str) -> Result<String, CaptchaError>;
type TopSolverGlobal = [OnceInit<TopSolverGlobalInner>; 6];
type CustomSolverGlobalInner = fn(&Agent, serde_json::Value, &str) -> Result<String, CaptchaError>;
type CustomSolverGlobal =
    OnceInit<Arc<RwLock<HashMap<&'static str, Box<CustomSolverGlobalInner>>>>>;
static TOP_SOLVER: TopSolverGlobal = [const { OnceInit::new() }; 6];
static CUSTOM_SOLVER: CustomSolverGlobal = OnceInit::new();
impl TopSolver {
    fn solver_generic<I, O, T>(
        agent: &Agent,
        image: serde_json::Value,
        referer: &str,
    ) -> Result<String, CaptchaError>
    where
        T: VerificationInfoTrait<I, O> + DeserializeOwned + 'static,
        SolverRaw<I, O>: 'static,
    {
        let self_: T = serde_json::from_value(image).unwrap();
        self_.solver(agent, referer)
    }
    const fn type_to_index(captcha_type: &CaptchaType) -> usize {
        match captcha_type {
            CaptchaType::Slide => 0,
            CaptchaType::TextClick => 1,
            CaptchaType::Rotate => 2,
            CaptchaType::IconClick => 3,
            CaptchaType::Obstacle => 4,
            CaptchaType::Custom(_) => unreachable!(),
        }
    }
    fn default_solver_impl(
        captcha_type: &CaptchaType,
    ) -> fn(&Agent, serde_json::Value, &str) -> Result<String, CaptchaError> {
        match captcha_type {
            CaptchaType::Slide => Self::solver_generic::<_, _, SlideImages>,
            CaptchaType::TextClick => Self::solver_generic::<_, _, TextClickInfo>,
            CaptchaType::Rotate => Self::solver_generic::<_, _, RotateImages>,
            CaptchaType::IconClick => Self::solver_generic::<_, _, IconClickImage>,
            CaptchaType::Obstacle => Self::solver_generic::<_, _, ObstacleImage>,
            CaptchaType::Custom(_) => unreachable!(),
        }
    }
    /// 该函数可以替换验证码枚举对应的验证信息类型为自定义实现。
    ///
    /// 需要 `T` 实现 [`VerificationInfoTrait`] 和 [`DeserializeOwned`]\(即可从 json 构造\), 且不能为临时类型。
    pub fn set_verification_info_type<T, I, O>(
        captcha_type: &CaptchaType,
    ) -> Result<(), OnceInitError>
    where
        T: VerificationInfoTrait<I, O> + DeserializeOwned + 'static,
        SolverRaw<I, O>: 'static,
    {
        match captcha_type {
            CaptchaType::Custom(r#type) => match CUSTOM_SOLVER.get_data() {
                Ok(map) => {
                    let mut map = map.write().unwrap();
                    if map.contains_key(r#type) {
                        Err(OnceInitError::DataInitialized)
                    } else {
                        map.insert(r#type, Box::new(Self::solver_generic::<_, _, T>));
                        Ok(())
                    }
                }
                Err(_) => {
                    let mut map = HashMap::<&'static str, Box<CustomSolverGlobalInner>>::new();
                    map.insert(r#type, Box::new(Self::solver_generic::<_, _, T>));
                    let map = Arc::new(RwLock::new(map));
                    CUSTOM_SOLVER.set_boxed_data(Box::new(map))
                }
            },
            t => TOP_SOLVER[Self::type_to_index(t)]
                .set_boxed_data(Box::new(Self::solver_generic::<_, _, T>)),
        }
    }
    pub fn solver(
        agent: &Agent,
        captcha_type: &CaptchaType,
        image: serde_json::Value,
        referer: &str,
    ) -> Result<String, CaptchaError> {
        match captcha_type {
            CaptchaType::Custom(r#type) => CUSTOM_SOLVER
                .get_data()
                .ok()
                .and_then(|map| {
                    map.read()
                        .unwrap()
                        .get(r#type)
                        .map(|a| a(agent, image, referer))
                })
                .ok_or_else(|| CaptchaError::UnsupportedType)?,
            t => match TOP_SOLVER[Self::type_to_index(t)].get_data() {
                Err(_) => Self::default_solver_impl(t)(agent, image, referer),
                Ok(solver_) => solver_(agent, image, referer),
            },
        }
    }
}
pub trait VerificationInfoTrait<I, O>: Sized {
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
    /// 设置全局过验证算法所需。
    ///
    /// 若输入输出类型（`I`, `O`）与默认类型如 [`SlideImages`] 等一致，实现该函数时，你可以直接调用默认类型的实现。
    ///
    /// 若不一致，你需要自定义一个 static 的 [`OnceInit<SolverRaw<I, O>>`](OnceInit) 数据。
    /// 另见 [`SolverRaw`].
    fn static_solver_holder() -> &'static OnceInit<SolverRaw<I, O>>;
    /// 将结果转为字符串类型，用来向网站发送请求。
    fn result_to_string(result: O) -> String;
    fn solve(input: I) -> Result<O, CaptchaError>
    where
        Self: VerificationInfoTrait<I, O> + 'static,
        SolverRaw<I, O>: 'static,
    {
        match Self::static_solver_holder().get_data() {
            Err(_) => Self::default_solver(input),
            Ok(s) => s(input),
        }
    }
    /// 初始化 `Solver`.
    ///
    /// 另见 [`VerificationInfoTrait::init_owned_solver`].
    fn init_solver<F: Fn(I) -> Result<O, CaptchaError> + Sync>(
        solver: &'static F,
    ) -> Result<(), OnceInitError>
    where
        I: 'static,
        O: 'static,
    {
        Self::static_solver_holder().set_data(solver)
    }
    /// 初始化 `Solver`.
    ///
    /// 另见 [`VerificationInfoTrait::init_solver`].
    fn init_owned_solver<F: Fn(I) -> Result<O, CaptchaError> + Sync + 'static>(
        solver: F,
    ) -> Result<(), OnceInitError>
    where
        I: 'static,
        O: 'static,
        Self: VerificationInfoTrait<I, O>,
    {
        let solver: Box<dyn Fn(_) -> _ + Sync + 'static> = Box::new(solver);
        Self::static_solver_holder().set_boxed_data(solver)
    }
    fn solver(self, agent: &Agent, referer: &str) -> Result<String, CaptchaError>
    where
        Self: 'static,
        SolverRaw<I, O>: 'static,
    {
        let data = self.prepare_data(agent, referer)?;
        let output = Self::solve(data)?;
        let r = Self::result_to_string(output);
        Ok(r)
    }
}
/// 类型别名，三个一组的 [`Point`] 类型。
pub type TriplePoint<T> = (Point<T>, Point<T>, Point<T>);
/// 类型别名，本质上是一个 `dyn Fn` 类型。
pub type SolverRaw<I, O> = dyn Fn(I) -> Result<O, CaptchaError> + Sync;
type SlideSolverRaw = SolverRaw<(DynamicImage, DynamicImage), u32>;
type IconClickSolverRaw = SolverRaw<DynamicImage, TriplePoint<u32>>;
type TextClickSolverRaw = SolverRaw<(String, DynamicImage), TriplePoint<u32>>;
type RotateSolverRaw = SolverRaw<(DynamicImage, DynamicImage), f32>;
type ObstacleSolverRaw = SolverRaw<DynamicImage, Point<u32>>;
static SLIDE_SOLVER: OnceInit<SlideSolverRaw> = OnceInit::new();
static ICON_CLICK_SOLVER: OnceInit<IconClickSolverRaw> = OnceInit::new();
static TEXT_CLICK_SOLVER: OnceInit<TextClickSolverRaw> = OnceInit::new();
static ROTATE_SOLVER: OnceInit<RotateSolverRaw> = OnceInit::new();
static OBSTACLE_SOLVER: OnceInit<ObstacleSolverRaw> = OnceInit::new();
fn download_image(
    agent: &Agent,
    image_url: &str,
    referer: &str,
) -> Result<DynamicImage, Box<ureq::Error>> {
    Ok(image_from_bytes(ureq_get_bytes(agent, image_url, referer)?))
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
    fn default_solver(
        (big_image, small_image): (DynamicImage, DynamicImage),
    ) -> Result<u32, CaptchaError> {
        time_it_and_print_result(move || {
            Ok(find_sub_image(
                &big_image,
                &small_image,
                cxlib_imageproc::slide_solvers::find_min_sum_of_squared_errors,
            ))
        })
    }
    fn static_solver_holder() -> &'static OnceInit<SlideSolverRaw> {
        &SLIDE_SOLVER
    }
    fn result_to_string(result: u32) -> String {
        debug!("本地滑块结果：{result}");
        format!("%5B%7B%22x%22%3A{}%7D%5D", result)
    }
}
impl VerificationInfoTrait<DynamicImage, TriplePoint<u32>> for IconClickImage {
    fn prepare_data(self, agent: &Agent, referer: &str) -> Result<DynamicImage, CaptchaError> {
        let img = download_image(agent, self.image_url(), referer)?;
        Ok(img)
    }
    fn static_solver_holder() -> &'static OnceInit<IconClickSolverRaw> {
        &ICON_CLICK_SOLVER
    }
    /// \[{"x":82,"y":114},{"x":286,"y":68},{"x":154,"y":90}\] <br/>
    /// x, y 为图标相对 origin_image 右上角的位置。
    fn result_to_string(result: TriplePoint<u32>) -> String {
        let points = click_captcha_helper::Point3::from((result.0, result.1, result.2));
        debug!("本地滑块结果：{points}");
        points.to_string()
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
    fn static_solver_holder() -> &'static OnceInit<TextClickSolverRaw> {
        &TEXT_CLICK_SOLVER
    }
    // TODO: 需要验证
    /// \[{"x":82,"y":114},{"x":286,"y":68},{"x":154,"y":90}\] <br/>
    /// x, y 为图标相对 origin_image 右上角的位置。
    fn result_to_string(result: TriplePoint<u32>) -> String {
        IconClickImage::result_to_string(result)
    }
}
impl VerificationInfoTrait<DynamicImage, Point<u32>> for ObstacleImage {
    fn prepare_data(self, agent: &Agent, referer: &str) -> Result<DynamicImage, CaptchaError> {
        debug!("图片 url：{}", self.img_url());
        let img = download_image(agent, self.img_url(), referer)?;
        Ok(img)
    }
    fn static_solver_holder() -> &'static OnceInit<ObstacleSolverRaw> {
        &OBSTACLE_SOLVER
    }
    // TODO: 需要验证
    /// \[{"x":82,"y":114},{"x":286,"y":68},{"x":154,"y":90}\] <br/>
    /// x, y 为图标相对 origin_image 右上角的位置。
    fn result_to_string(result: Point<u32>) -> String {
        let data = click_captcha_helper::Point::from(result);
        debug!("本地滑块结果：{data}");
        format!("%5B{}%5D", data)
    }
}
impl VerificationInfoTrait<(DynamicImage, DynamicImage), f32> for RotateImages {
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
        let rotatable_img = download_image(agent, self.rotatable_img_url(), referer)?;
        let fixed_img = download_image(agent, self.fixed_img_url(), referer)?;
        Ok((fixed_img, rotatable_img))
    }
    fn static_solver_holder() -> &'static OnceInit<RotateSolverRaw> {
        &ROTATE_SOLVER
    }
    /// result 为度数，取值为 0-504.
    fn result_to_string(result: f32) -> String {
        debug!("本地旋转结果：{result}");
        let result = ((result / 504.0 * 280.0).round() as u32).to_string();
        format!("%5B%7B%22x%22%3A{}%7D%5D", result)
    }
}
