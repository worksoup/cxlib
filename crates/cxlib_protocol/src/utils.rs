use cxlib_error::AgentError;
use cxlib_error_utils::CxlibResultUtils;
use log::debug;
use std::ops::{Deref, DerefMut};
use ureq::{Agent, Body, http::Response};

pub struct PPTSignHelper {
    url: String,
}
impl PPTSignHelper {
    #[inline]
    pub fn url(&self) -> &str {
        &self.url
    }
    #[inline]
    pub fn get(&self, agent: &Agent) -> Result<Response<Body>, AgentError> {
        Ok(agent.get(self.url()).call()?)
    }
    #[inline]
    pub fn with_enc2(mut self, enc2: &str) -> Self {
        self.url += "&enc2=";
        self.url += enc2;
        self
    }
    #[inline]
    pub fn with_validate(mut self, validate: &str) -> Self {
        self.url += "&validate=";
        self.url += validate;
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
impl Deref for PPTSignHelper {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.url()
    }
}
impl DerefMut for PPTSignHelper {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.url
    }
}

impl From<String> for PPTSignHelper {
    #[inline]
    fn from(s: String) -> Self {
        Self { url: s }
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
