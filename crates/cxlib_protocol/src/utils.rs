use cxlib_error::AgentError;
use cxlib_error_utils::CxlibResultUtils;
use log::debug;
use std::{borrow::Cow, ops::Deref};
use ureq::{Agent, Body, http::Response};

pub trait SignHelperTrait: Sized {
    fn url(self) -> Cow<'static, str>;
    // 签到
    #[inline]
    fn general_sign_url(
        self,
        (uid, fid, stu_name): (&str, &str, &str),
        active_id: &str,
    ) -> crate::utils::SignUrlHelper {
        let url = self.url();
        let param = format!(
            "activeId={active_id}&uid={uid}&clientip=&latitude=-1&longitude=-1&appType=15&fid={fid}&name={stu_name}"
        );
        SignUrlHelper { url, param }
    }
    #[inline]
    fn photo_sign_url(
        self,
        (uid, fid, stu_name): (&str, &str, &str),
        active_id: &str,
        object_id: &str,
    ) -> crate::utils::SignUrlHelper {
        // NOTE 存疑。
        let url = self.url();
        let param = format!(
            "activeId={active_id}&uid={uid}&clientip=&useragent=&latitude=-1&longitude=-1&appType=15&fid={fid}&objectId={object_id}&name={}",
            percent_encoding::utf8_percent_encode(stu_name, percent_encoding::NON_ALPHANUMERIC)
        );
        SignUrlHelper { url, param }
    }

    fn qrcode_sign_url(
        self,
        (uid, fid, stu_name): (&str, &str, &str),
        enc: &str,
        active_id: &str,
        location: Option<&cxlib_base_types::Geoaddr>,
    ) -> crate::utils::SignUrlHelper {
        let url = self.url();
        // TODO: 存疑。
        let param = if let Some(addr) = location {
            let (addr, geolocation) = addr.decompose_as();
            let (lon, lat, alt) = {
                let l = geolocation;
                (l.lon(), l.lat(), l.alt())
            };
            let location_str = format!(
                r#"{{"result":"1","address":"{addr}","latitude":{lat},"longitude":{lon},"altitude":{alt}}}"#
            );
            let location_str = percent_encoding::utf8_percent_encode(
                &location_str,
                percent_encoding::NON_ALPHANUMERIC,
            )
            .to_string();
            format!(
                r#"enc={enc}&name={stu_name}&activeId={active_id}&uid={uid}&clientip=&location={location_str}&latitude=-1&longitude=-1&fid={fid}&appType=15"#
            )
        } else {
            format!(
                r#"enc={enc}&name={stu_name}&activeId={active_id}&uid={uid}&clientip=&location=&latitude=-1&longitude=-1&fid={fid}&appType=15"#
            )
        };
        SignUrlHelper { url, param }
    }
    #[inline]
    fn location_sign_url(
        self,
        (uid, fid, stu_name): (&str, &str, &str),
        (addr, lat, lon): (&str, &str, &str),
        active_id: &str,
        is_auto_location: bool,
    ) -> crate::utils::SignUrlHelper {
        let url = self.url();
        let if_tijiao = if is_auto_location { 1 } else { 0 };
        let param = format!(
            "name={stu_name}&address={addr}&activeId={active_id}&uid={uid}&clientip=&latitude={lat}&longitude={lon}&fid={fid}&appType=15&ifTiJiao={if_tijiao}"
        );
        SignUrlHelper { url, param }
    }

    #[inline]
    fn signcode_sign_url(
        self,
        (uid, fid, stu_name): (&str, &str, &str),
        active_id: &str,
        signcode: &str,
    ) -> crate::utils::SignUrlHelper {
        let url = self.url();
        let param = format!(
            "activeId={active_id}&uid={uid}&clientip=&latitude=-1&longitude=-1&appType=15&fid={fid}&name={stu_name}&signCode={signcode}"
        );
        SignUrlHelper { url, param }
    }
}
impl SignHelperTrait for &'static str {
    #[inline]
    fn url(self) -> Cow<'static, str> {
        self.into()
    }
}
impl SignHelperTrait for String {
    #[inline]
    fn url(self) -> Cow<'static, str> {
        self.into()
    }
}
impl SignHelperTrait for Cow<'static, str> {
    #[inline]
    fn url(self) -> Cow<'static, str> {
        self
    }
}
pub struct SignUrlHelper {
    url: Cow<'static, str>,
    param: String,
}
impl SignUrlHelper {
    #[inline]
    pub fn url(&self) -> &str {
        &self.url
    }
    #[inline]
    pub fn param(&self) -> &str {
        &self.param
    }
    #[inline]
    pub fn get(&self, agent: &Agent) -> Result<Response<Body>, AgentError> {
        Ok(agent
            .get(format!("{}?{}", self.url(), self.param()))
            .call()?)
    }
    #[inline]
    pub fn with_enc2(mut self, enc2: &str) -> Self {
        self.param += "&enc2=";
        self.param += enc2;
        self
    }
    #[inline]
    pub fn with_validate(mut self, validate: &str) -> Self {
        self.param += "&validate=";
        self.param += validate;
        self
    }
    #[inline]
    pub fn patch_enc_by_pre_sign_result_msg(self, msg: String) -> Self {
        if msg.len() > 9 {
            let enc2 = &msg[9..msg.len()];
            debug!("enc2: {enc2:?}");
            self.with_enc2(enc2)
        } else {
            self
        }
    }
}
impl Deref for SignUrlHelper {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.url()
    }
}

#[inline]
pub fn trim_response_to_json<R: std::io::Read, T>(mut reader: R) -> Result<T, serde_json::Error>
where
    T: serde::de::DeserializeOwned,
{
    let mut buf = String::new();
    let size = reader.read_to_string(&mut buf).log_unwrap();
    let s = &buf[crate::collect::CALLBACK_NAME.len() + 1..size - 1];
    debug!("{s}");
    serde_json::from_str(s)
}
