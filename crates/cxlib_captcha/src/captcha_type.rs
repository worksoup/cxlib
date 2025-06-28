use crate::{
    CaptchaError, VerificationInfoTrait,
    hash::{encode, hash, uuid},
    utils::{get_now_timestamp_mills, get_server_time, trim_response_to_json},
};
use cxlib_error::AgentError;
use cxlib_error_utils::{CxlibResultUtils, MaybeFatalError};
use cxlib_protocol::collect::CaptchaProtocolTrait;
use log::{debug, warn};
use serde::{Deserialize, de::DeserializeOwned};
use ureq::Agent;

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
pub trait CaptchaSolverTrait {
    fn solver(
        agent: &Agent,
        image: serde_json::Value,
        referer: &str,
    ) -> Result<String, CaptchaError>;
    fn generate_secrets(captcha_id: &str, server_time_stamp_mills: u128) -> (String, String);

    fn generate_iv(captcha_id: &str) -> String;
    fn get_captcha<CaptchaProtocol: CaptchaProtocolTrait>(
        agent: &Agent,
        captcha_id: &str,
        server_time_mills: u128,
        referer: &str,
    ) -> Result<GetCaptchaResult, AgentError>;
    fn check_captcha<CaptchaProtocol: CaptchaProtocolTrait>(
        agent: &Agent,
        captcha_id_iv_token: (&str, &str, &str),
        text_click_arr: &str,
        server_time_mills: u128,
    ) -> Result<String, CaptchaError>;
    fn solve_captcha<CaptchaProtocol: CaptchaProtocolTrait>(
        agent: &Agent,
        captcha_id: &str,
        referer: &str,
    ) -> Result<String, CaptchaError>;
}
impl<T> CaptchaSolverTrait for T
where
    T: VerificationInfoTrait + DeserializeOwned + 'static,
{
    fn solver(
        agent: &Agent,
        image: serde_json::Value,
        referer: &str,
    ) -> Result<String, CaptchaError> {
        let self_: Self = serde_json::from_value(image).unwrap();
        self_.solve(agent, referer)
    }
    fn generate_secrets(captcha_id: &str, server_time_stamp_mills: u128) -> (String, String) {
        let server_time_str = server_time_stamp_mills.to_string();
        let captcha_key = encode(hash(&(server_time_str.clone() + &uuid())));
        // "%3A" 即英文冒号的转义。
        let tmp_token = encode(hash(
            &(server_time_str + captcha_id + Self::captcha_type() + &captcha_key),
        )) + "%3A"
            + (server_time_stamp_mills + 300000_u128).to_string().as_str();
        (captcha_key, tmp_token)
    }

    fn generate_iv(captcha_id: &str) -> String {
        let iv_uuid = uuid();
        encode(hash(
            &(captcha_id.to_owned()
                + Self::captcha_type()
                + get_now_timestamp_mills().to_string().as_str()
                + &iv_uuid),
        ))
    }
    fn get_captcha<CaptchaProtocol: CaptchaProtocolTrait>(
        agent: &Agent,
        captcha_id: &str,
        server_time_mills: u128,
        referer: &str,
    ) -> Result<GetCaptchaResult, AgentError> {
        let (captcha_key, tmp_token) = Self::generate_secrets(captcha_id, server_time_mills);
        let iv = Self::generate_iv(captcha_id);
        let r = CaptchaProtocol::get_captcha(
            agent,
            Self::captcha_type(),
            captcha_id,
            (&captcha_key, &tmp_token),
            &iv,
            server_time_mills + 1,
            referer,
        )?;
        let r_data = trim_response_to_json(
            &r.into_body()
                .read_to_string()
                .expect("CaptchaResponse into String failed."),
        )
        .expect("Failed trim_response_to_json");
        Ok(GetCaptchaResult { iv, data: r_data })
    }
    fn check_captcha<CaptchaProtocol: CaptchaProtocolTrait>(
        agent: &Agent,
        (captcha_id, iv, token): (&str, &str, &str),
        text_click_arr: &str,
        server_time_mills: u128,
    ) -> Result<String, CaptchaError> {
        let r = CaptchaProtocol::check_captcha(
            agent,
            Self::captcha_type(),
            captcha_id,
            text_click_arr,
            token,
            iv,
            server_time_mills + 2,
        )?;
        let v: ValidateResult =
            trim_response_to_json(&r.into_body().read_to_string().log_unwrap()).log_unwrap();
        debug!("验证结果：{v:?}");
        v.get_validate_info()
    }
    fn solve_captcha<CaptchaProtocol: CaptchaProtocolTrait>(
        agent: &Agent,
        captcha_id: &str,
        referer: &str,
    ) -> Result<String, CaptchaError> {
        let local_time = get_now_timestamp_mills();
        let server_time = get_server_time::<CaptchaProtocol>(agent, captcha_id, local_time)?;
        // 事不过三。
        for i in 0..3 {
            match Self::get_captcha::<CaptchaProtocol>(agent, captcha_id, server_time + i, referer)
                .map_err(CaptchaError::from)
                .and_then(
                    |GetCaptchaResult {
                         iv,
                         data: VerificationDataWithToken { token, data },
                     }| {
                        Self::solver(agent, data, referer).and_then(|text_click_arr| {
                            Self::check_captcha::<CaptchaProtocol>(
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
    use crate::{
        CaptchaSolverTrait, IconClickImage, ObstacleImage, RotateImages, SlideImages,
        TextClickInfo, VerificationInfoTrait,
        hash::{encode, hash},
        utils::{get_now_timestamp_mills, get_server_time},
    };
    use cxlib_protocol::collect::{CAPTCHA_ID, CaptchaProtocol};
    use serde::de::DeserializeOwned;

    const REFERER: &str = "https%3A%2F%2Fmobilelearn.chaoxing.com";
    #[test]
    fn auto_solve_captcha_test() {
        let agent = ureq::Agent::new_with_defaults();
        let r = RotateImages::solve_captcha::<CaptchaProtocol>(&agent, CAPTCHA_ID, REFERER);
        println!("{:?}", r);
    }
    #[test]
    fn generate_captcha_key() {
        fn assert_eq_with_real_value<T>(
            real_value: &str,
            server_time: u128,
            captcha_id: &str,
            captcha_key: &str,
        ) where
            T: VerificationInfoTrait + DeserializeOwned + 'static,
        {
            let tmp_token = encode(hash(
                &(server_time.to_string() + captcha_id + T::captcha_type() + captcha_key),
            )) + "%3A"
                + (server_time + 300000_u128).to_string().as_str();
            assert_eq!(real_value, tmp_token);
        }
        assert_eq_with_real_value::<IconClickImage>(
            "21d29919dc55f9a25b25a9aec531682e%3A1733129174649",
            1733128874649,
            CAPTCHA_ID,
            "0062a52fa1d93307b2bc503883986cf9",
        )
    }
    #[test]
    fn get_captcha_test() {
        fn get_captcha_<T>()
        where
            T: VerificationInfoTrait + DeserializeOwned + 'static,
        {
            let agent = ureq::Agent::new_with_defaults();
            let local_time = get_now_timestamp_mills();
            let captcha_id = CAPTCHA_ID;
            let server_time =
                get_server_time::<CaptchaProtocol>(&agent, captcha_id, local_time).unwrap();
            let validate_info =
                T::get_captcha::<CaptchaProtocol>(&agent, captcha_id, server_time + 1, REFERER)
                    .unwrap();
            println!("{:?}", validate_info);
        }
        get_captcha_::<IconClickImage>();
        get_captcha_::<ObstacleImage>();
        get_captcha_::<RotateImages>();
        get_captcha_::<SlideImages>();
        get_captcha_::<TextClickInfo>();
    }
}
