use log::{debug, warn};
use serde::Deserialize;

use crate::hash::{encode, hash, uuid};
use crate::protocol::{check_captcha, get_captcha, get_server_time};

// Doesn't matter.
static CALLBACK_NAME: &str = "jQuery_114514_1919810";

pub fn trim_response_to_json<'a, T>(text: &'a str) -> Result<T, ureq::serde_json::Error>
where
    T: ureq::serde::de::Deserialize<'a>,
{
    let s = &text[CALLBACK_NAME.len() + 1..text.len() - 1];
    debug!("{s}");
    ureq::serde_json::from_str(s)
}

pub fn generate_secrets(
    captcha_id: &str,
    server_time_stamp_mills: u128,
    r#type: &str,
) -> (String, String) {
    let captcha_key = encode(hash(&(server_time_stamp_mills.to_string() + &uuid())));
    let tmp_token = encode(hash(
        &(server_time_stamp_mills.to_string() + captcha_id + r#type + &captcha_key),
    ));
    let tmp_token =
        tmp_token + "%3A" + (server_time_stamp_mills + 300000_u128).to_string().as_str();
    (captcha_key, tmp_token)
}

pub fn auto_solve_captcha(
    agent: &ureq::Agent,
    captcha_id: &str,
    time: u128,
) -> Result<ValidateResult, Box<ureq::Error>> {
    let (key, tmp_token) = generate_secrets(captcha_id, time, "slide");
    let r = get_captcha(agent, captcha_id, &key, &tmp_token, time + 1)?;
    #[derive(Deserialize)]
    struct Images {
        #[serde(rename = "shadeImage")]
        shade_image_url: String,
        #[serde(rename = "cutoutImage")]
        cutout_image_url: String,
    }
    #[derive(Deserialize)]
    struct Tmp {
        token: String,
        #[serde(rename = "imageVerificationVo")]
        images: Images,
    }
    let Tmp {
        token,
        images: Images {
            shade_image_url,
            cutout_image_url,
        },
    } = trim_response_to_json(&r.into_string().unwrap()).unwrap();
    debug!("滑块图片 url：{}, {}", shade_image_url, cutout_image_url);
    let agent = ureq::Agent::new();
    let small_img = cxsign_imageproc::download_image(&agent, &cutout_image_url)?;
    let big_img = cxsign_imageproc::download_image(&agent, &shade_image_url)?;
    let max_x = cxsign_imageproc::find_sub_image(&big_img, &small_img);
    debug!("本地滑块结果：{max_x}");
    let r = check_captcha(&agent, captcha_id, max_x, &token, time + 2)?;
    let v: ValidateResult = trim_response_to_json(&r.into_string().unwrap()).unwrap();
    debug!("滑块结果：{v:?}");
    Ok(v)
}

pub fn captcha_solver(
    agent: &ureq::Agent,
    captcha_id: &str,
) -> Result<String, cxsign_error::Error> {
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let r = get_server_time(agent, captcha_id, time)?;
    #[derive(Deserialize)]
    struct Tmp {
        t: u128,
    }
    let Tmp { t } = trim_response_to_json(r.into_string().unwrap().as_str()).unwrap();
    // 事不过三。
    for i in 0..3 {
        if let Some(c) = auto_solve_captcha(agent, captcha_id, t + i)?.get_validate_info() {
            return Ok(c);
        } else {
            warn!("滑块验证失败，即将重试。")
        }
    }
    Err(cxsign_error::Error::CaptchaEmptyError)
}

#[derive(Deserialize, Debug)]
pub struct ValidateResult {
    #[serde(rename = "extraData")]
    extra_data: Option<String>,
}

impl ValidateResult {
    pub fn get_validate_info(&self) -> Option<String> {
        #[derive(Deserialize)]
        struct Tmp {
            validate: String,
        }
        self.extra_data.as_ref().map(|s| {
            debug!("{s}");
            let Tmp { validate } = ureq::serde_json::from_str(s).unwrap();
            validate
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::protocol::{get_server_time, CAPTCHA_ID};
    use crate::utils::{auto_solve_captcha, trim_response_to_json};
    use serde::Deserialize;

    #[test]
    fn auto_solve_captcha_test() {
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        println!("{time}");
        let agent = ureq::Agent::new();
        let r = get_server_time(&agent, CAPTCHA_ID, time).unwrap();
        #[derive(Deserialize)]
        struct Tmp {
            t: u128,
        }
        let Tmp { t } = trim_response_to_json(r.into_string().unwrap().as_str()).unwrap();
        let r = auto_solve_captcha(&agent, CAPTCHA_ID, t).unwrap();
        println!("{:?}", r.get_validate_info());
    }
}
