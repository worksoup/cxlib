use crate::{
    hash::{encode, hash, uuid},
    protocol::{check_captcha, get_captcha},
    utils::{get_now_timestamp_mills, get_server_time, trim_response_to_json},
    TopSolver, DEFAULT_CAPTCHA_TYPE,
};
use cxlib_error::{CaptchaError, UnwrapOrLogPanic};
use log::{debug, warn};
use onceinit::{OnceInitError, StaticDefault};
use serde::Deserialize;
use std::fmt::{Display, Formatter};
use ureq::{serde_json, Agent, Error};

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
/// 如需自行支持，请为该类型 [实现 `Solver`](crate::solver::VerificationInfoTrait::init_solver).
///
/// 若需自行处理图片下载等步骤，参见 [`TopSolver::set_verification_info_type`].
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
}
unsafe impl StaticDefault for CaptchaType {
    fn static_default() -> &'static Self {
        &CaptchaType::Slide
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
        }
    }
}
impl CaptchaType {
    /// 将当前验证码类型设为全局默认。
    ///
    /// 注意，默认类型仅可设置一次。
    pub fn as_global_default(&self) -> Result<(), OnceInitError> {
        DEFAULT_CAPTCHA_TYPE.set_boxed_data(Box::new(self.clone()))
    }
    /// 将设置全局默认的验证码类型。
    ///
    /// 注意，默认类型仅可设置一次。
    pub fn set_global_default(self_: &CaptchaType) -> Result<(), OnceInitError> {
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
    ) -> Result<GetCaptchaResult, Box<Error>> {
        let (captcha_key, tmp_token) = self.generate_secrets(captcha_id, server_time_mills);
        let iv = self.generate_iv(captcha_id);
        let r = get_captcha(
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
        let r = check_captcha(
            agent,
            self,
            captcha_id,
            text_click_arr,
            token,
            iv,
            server_time_mills + 2,
        )?;
        let v: ValidateResult =
            trim_response_to_json(&r.into_string().unwrap_or_log_panic()).unwrap();
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
                        TopSolver::solver(agent, self, data, referer).and_then(|text_click_arr| {
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
                Err(e) => match e {
                    r @ CaptchaError::Canceled(_) => {
                        return Err(r);
                    }
                    _ => warn!("滑块验证失败：{e}，即将重试。"),
                },
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
        let r =
            CaptchaType::Slide.solve_captcha(&agent, &ProtocolItem::CaptchaId.to_string(), REFERER);
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
