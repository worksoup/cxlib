use crate::{
    hash::{encode, hash, uuid},
    utils::{get_now_timestamp_mills, get_server_time, trim_response_to_json},
    IconClickImage, ObstacleImage, RotateImages, SlideImages, SolverRaw, TextClickInfo,
    VerificationInfoTrait, DEFAULT_CAPTCHA_TYPE,
};
use cxlib_error::{AgentError, CaptchaError, CxlibResultUtils, InitError, MaybeFatalError};
use cxlib_protocol::collect::captcha as protocol;
use log::{debug, warn};
use onceinit::{OnceInit, OnceInitError, StaticDefault};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use ureq::{serde_json, Agent};

#[derive(Debug)]
pub struct GetCaptchaResult {
    pub iv: String,
    pub data: VerificationDataWithToken,
}
#[derive(Debug, Deserialize)]
pub struct VerificationDataWithToken {
    pub token: String,
    #[serde(rename = "imageVerificationVo")]
    pub data: serde_json::Value,
}

#[derive(Deserialize, Debug)]
pub struct ValidateResult {
    #[serde(rename = "extraData")]
    extra_data: Option<String>,
}

impl ValidateResult {
    pub fn get_validate_info(&self) -> Result<String, CaptchaError> {
        #[derive(Deserialize)]
        struct Tmp {
            validate: String,
        }
        self.extra_data
            .as_ref()
            .map(|s| {
                debug!("{s}");
                let Tmp { validate } = serde_json::from_str(s).unwrap();
                validate
            })
            .ok_or_else(|| CaptchaError::VerifyFailed)
    }
}
/// # [`CaptchaType`]
/// 验证码类型，目前只有 [`CaptchaType::Slide`] 类型支持良好，无需初始化 `Solver`.
/// 如需自行支持，请为该类型 [实现 `Solver`](CaptchaType::init_solver).
///
/// 若需自行处理图片下载等步骤，参见 [`CaptchaType::set_verification_info_type`].
/// 该函数可以替换掉默认的验证信息类型。
#[derive(Debug, Clone)]
pub enum CaptchaType {
    /// ## 滑块验证码
    /// 拖动滑块至合适位置，完成验证。
    ///
    /// 对应的验证信息类型为 [`SlideImages`],
    /// 如需自定义 `Solver`, 请参考其文档。
    Slide,
    /// ## 文字点选验证码
    /// 按照提示依次点击三个汉字，完成验证。
    ///
    /// 对应的验证信息类型为 [`TextClickInfo`],
    /// 请参考其文档初始化 `Solver`.
    TextClick,
    /// ## 图片旋转验证码
    /// 将图片旋转至合适角度，完成验证。
    ///
    /// 对应的验证信息类型为 [`RotateImages`],
    /// 请参考其文档初始化 `Solver`.
    Rotate,
    /// ## 图标点选验证码
    /// 按照提示依次点击三个图标，完成验证。
    ///
    /// 对应的验证信息类型为 [`IconClickImage`],
    /// 请参考其文档初始化 `Solver`.
    IconClick,
    /// ## 单图标点选验证码
    /// 按照提示点击单个图标，完成验证。
    ///
    /// 对应的验证信息类型为 [`ObstacleImage`],
    /// 请参考其文档初始化 `Solver`.
    Obstacle,
    /// ## 自定义类型验证码
    Custom(&'static str),
}
impl CaptchaType {
    const DEFAULT: CaptchaType = CaptchaType::Rotate;
}
impl Default for CaptchaType {
    fn default() -> Self {
        Self::DEFAULT
    }
}
unsafe impl StaticDefault for CaptchaType {
    fn static_default() -> &'static Self {
        &CaptchaType::DEFAULT
    }
}
impl FromStr for CaptchaType {
    type Err = CaptchaError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "slide" | "Slide" | "SLIDE" => CaptchaType::Slide,
            "iconclick" | "IconClick" | "ICONCLICK" | "icon_click" | "ICON_CLICK" => {
                CaptchaType::IconClick
            }
            "textclick" | "TextClick" | "TEXTCLICK" | "text_click" | "TEXT_CLICK" => {
                CaptchaType::TextClick
            }
            "obstacle" | "Obstacle" | "OBSTACLE" => CaptchaType::Obstacle,
            "rotate" | "Rotate" | "ROTATE" => CaptchaType::Rotate,
            _ => Err(CaptchaError::UnsupportedType)?,
        })
    }
}
impl Display for CaptchaType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}
impl AsRef<str> for CaptchaType {
    fn as_ref(&self) -> &str {
        match self {
            CaptchaType::Slide => "slide",
            CaptchaType::TextClick => "textclick",
            CaptchaType::Rotate => "rotate",
            CaptchaType::IconClick => "iconclick",
            CaptchaType::Obstacle => "obstacle",
            CaptchaType::Custom(r#type) => r#type,
        }
    }
}
type TopSolverGlobalInner = fn(&Agent, serde_json::Value, &str) -> Result<String, CaptchaError>;
type TopSolverGlobal = [OnceInit<TopSolverGlobalInner>; 6];
type CustomSolverGlobalInner = fn(&Agent, serde_json::Value, &str) -> Result<String, CaptchaError>;
type CustomSolverGlobal =
    OnceInit<Arc<RwLock<HashMap<&'static str, Box<CustomSolverGlobalInner>>>>>;
static TOP_SOLVER: TopSolverGlobal = [const { OnceInit::uninit() }; 6];
static CUSTOM_SOLVER: CustomSolverGlobal = OnceInit::uninit();
impl CaptchaType {
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
    const fn type_to_index(&self) -> usize {
        match self {
            CaptchaType::Slide => 0,
            CaptchaType::TextClick => 1,
            CaptchaType::Rotate => 2,
            CaptchaType::IconClick => 3,
            CaptchaType::Obstacle => 4,
            CaptchaType::Custom(_) => unreachable!(),
        }
    }
    fn default_solver_impl(
        &self,
    ) -> fn(&Agent, serde_json::Value, &str) -> Result<String, CaptchaError> {
        match self {
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
    pub fn set_verification_info_type<T, I, O>(&self) -> Result<(), InitError>
    where
        T: VerificationInfoTrait<I, O> + DeserializeOwned + 'static,
        SolverRaw<I, O>: 'static,
    {
        match self {
            CaptchaType::Custom(r#type) => match CUSTOM_SOLVER.get() {
                Ok(map) => {
                    let mut map = map.write().unwrap();
                    if map.contains_key(r#type) {
                        Err(OnceInitError::DataInitialized)?
                    } else {
                        map.insert(r#type, Box::new(Self::solver_generic::<_, _, T>));
                        Ok(())
                    }
                }
                Err(_) => {
                    let mut map = HashMap::<&'static str, Box<CustomSolverGlobalInner>>::new();
                    map.insert(r#type, Box::new(Self::solver_generic::<_, _, T>));
                    let map = Arc::new(RwLock::new(map));
                    Ok(CUSTOM_SOLVER.init_boxed(Box::new(map))?)
                }
            },
            t => Ok(TOP_SOLVER[Self::type_to_index(t)]
                .init_boxed(Box::new(Self::solver_generic::<_, _, T>))?),
        }
    }
    /// 初始化 `Solver`.
    ///
    /// 另见 [`VerificationInfoTrait::init_solver`].
    pub fn init_solver<T: VerificationInfoTrait<I, O>, I, O>(
        solver: &'static (impl Fn(I) -> Result<O, CaptchaError> + Sync),
    ) -> Result<(), InitError>
    where
        I: 'static,
        O: 'static,
    {
        T::init_solver(solver)
    }
    /// 初始化 `Solver`.
    ///
    /// 另见 [`VerificationInfoTrait::init_owned_solver`].
    pub fn init_owned_solver<T: VerificationInfoTrait<I, O>, I, O>(
        solver: impl Fn(I) -> Result<O, CaptchaError> + Sync + 'static,
    ) -> Result<(), InitError>
    where
        I: 'static,
        O: 'static,
    {
        T::init_owned_solver(solver)
    }
    pub fn solver(
        &self,
        agent: &Agent,
        image: serde_json::Value,
        referer: &str,
    ) -> Result<String, CaptchaError> {
        match self {
            CaptchaType::Custom(r#type) => CUSTOM_SOLVER
                .get()
                .ok()
                .and_then(|map| {
                    map.read()
                        .unwrap()
                        .get(r#type)
                        .map(|a| a(agent, image, referer))
                })
                .ok_or_else(|| CaptchaError::UnsupportedType)?,
            t => match TOP_SOLVER[Self::type_to_index(t)].get() {
                Err(_) => Self::default_solver_impl(t)(agent, image, referer),
                Ok(solver_) => solver_(agent, image, referer),
            },
        }
    }
}
impl CaptchaType {
    /// 将当前验证码类型设为全局默认。
    ///
    /// 注意，默认类型仅可设置一次。
    pub fn as_global_default(&self) -> Result<(), InitError> {
        Ok(DEFAULT_CAPTCHA_TYPE.init_boxed(Box::new(self.clone()))?)
    }
    /// 将设置全局默认的验证码类型。
    ///
    /// 注意，默认类型仅可设置一次。
    pub fn set_global_default(self_: &CaptchaType) -> Result<(), InitError> {
        self_.as_global_default()
    }
    pub fn generate_secrets(
        &self,
        captcha_id: &str,
        server_time_stamp_mills: u128,
    ) -> (String, String) {
        let server_time_str = server_time_stamp_mills.to_string();
        let captcha_key = encode(hash(&(server_time_str.clone() + &uuid())));
        // "%3A" 即英文冒号的转义。
        let tmp_token = encode(hash(
            &(server_time_str + captcha_id + self.as_ref() + &captcha_key),
        )) + "%3A"
            + (server_time_stamp_mills + 300000_u128).to_string().as_str();
        (captcha_key, tmp_token)
    }

    pub fn generate_iv(&self, captcha_id: &str) -> String {
        let iv_uuid = uuid();
        let iv = encode(hash(
            &(captcha_id.to_owned()
                + self.as_ref()
                + get_now_timestamp_mills().to_string().as_str()
                + &iv_uuid),
        ));
        iv
    }
    pub fn get_captcha(
        &self,
        agent: &Agent,
        captcha_id: &str,
        server_time_mills: u128,
        referer: &str,
    ) -> Result<GetCaptchaResult, AgentError> {
        let (captcha_key, tmp_token) = self.generate_secrets(captcha_id, server_time_mills);
        let iv = self.generate_iv(captcha_id);
        let r = protocol::get_captcha(
            agent,
            self,
            captcha_id,
            (&captcha_key, &tmp_token),
            &iv,
            server_time_mills + 1,
            referer,
        )?;
        let r_data = trim_response_to_json(
            &r.into_string()
                .expect("CaptchaResponse into String failed."),
        )
        .expect("Failed trim_response_to_json");
        Ok(GetCaptchaResult { iv, data: r_data })
    }
    pub fn check_captcha(
        &self,
        agent: &Agent,
        (captcha_id, iv, token): (&str, &str, &str),
        text_click_arr: &str,
        server_time_mills: u128,
    ) -> Result<String, CaptchaError> {
        let r = protocol::check_captcha(
            agent,
            self,
            captcha_id,
            text_click_arr,
            token,
            iv,
            server_time_mills + 2,
        )?;
        let v: ValidateResult = trim_response_to_json(&r.into_string().log_unwrap()).log_unwrap();
        debug!("验证结果：{v:?}");
        v.get_validate_info()
    }
    pub fn solve_captcha(
        &self,
        agent: &Agent,
        captcha_id: &str,
        referer: &str,
    ) -> Result<String, CaptchaError> {
        let local_time = get_now_timestamp_mills();
        let server_time = get_server_time(agent, captcha_id, local_time)?;
        // 事不过三。
        for i in 0..3 {
            match self
                .get_captcha(agent, captcha_id, server_time + i, referer)
                .map_err(CaptchaError::from)
                .and_then(
                    |GetCaptchaResult {
                         iv,
                         data: VerificationDataWithToken { token, data },
                     }| {
                        self.solver(agent, data, referer)
                            .and_then(|text_click_arr| {
                                Self::check_captcha(
                                    self,
                                    agent,
                                    (captcha_id, &iv, &token),
                                    &text_click_arr,
                                    server_time + i,
                                )
                            })
                    },
                ) {
                r @ Ok(_) => {
                    return r;
                }
                Err(e) => {
                    if e.is_fatal() {
                        return Err(e);
                    } else {
                        warn!("滑块验证失败：{e}，即将重试。");
                    }
                }
            }
        }
        Err(CaptchaError::VerifyFailed)
    }
}

#[cfg(test)]
mod tests {
    use crate::hash::{encode, hash};
    use crate::utils::{get_now_timestamp_mills, get_server_time};
    use crate::CaptchaType;
    use cxlib_protocol::{ProtocolItem, ProtocolItemTrait};

    const REFERER: &str = "https%3A%2F%2Fmobilelearn.chaoxing.com";
    #[test]
    fn auto_solve_captcha_test() {
        let agent = ureq::Agent::new();
        let r = CaptchaType::Rotate.solve_captcha(
            &agent,
            &ProtocolItem::CaptchaId.to_string(),
            REFERER,
        );
        println!("{:?}", r);
    }
    #[test]
    fn generate_captcha_key() {
        fn assert_eq_with_real_value(
            real_value: &str,
            server_time: u128,
            captcha_id: &str,
            captcha_type: CaptchaType,
            captcha_key: &str,
        ) {
            let tmp_token = encode(hash(
                &(server_time.to_string() + captcha_id + captcha_type.as_ref() + captcha_key),
            )) + "%3A"
                + (server_time + 300000_u128).to_string().as_str();
            assert_eq!(real_value, tmp_token);
        }
        assert_eq_with_real_value(
            "21d29919dc55f9a25b25a9aec531682e%3A1733129174649",
            1733128874649,
            ProtocolItem::CaptchaId.get().as_ref(),
            CaptchaType::IconClick,
            "0062a52fa1d93307b2bc503883986cf9",
        )
    }
    #[test]
    fn get_captcha_test() {
        fn get_captcha_(captcha_type: CaptchaType) {
            let agent = ureq::Agent::new();
            let local_time = get_now_timestamp_mills();
            let captcha_id = ProtocolItem::CaptchaId.get();
            let server_time = get_server_time(&agent, captcha_id.as_ref(), local_time).unwrap();
            let validate_info = captcha_type
                .get_captcha(&agent, captcha_id.as_ref(), server_time + 1, REFERER)
                .unwrap();
            println!("{:?}", validate_info);
        }
        get_captcha_(CaptchaType::IconClick);
        get_captcha_(CaptchaType::Obstacle);
        get_captcha_(CaptchaType::Rotate);
        get_captcha_(CaptchaType::Slide);
        get_captcha_(CaptchaType::TextClick);
    }
}
