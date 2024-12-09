use std::ops::{Deref, DerefMut};
use log::debug;
use ureq::{Agent, Response};
use cxlib_error::AgentError;

pub struct PPTSignHelper {
    url: String,
}
impl PPTSignHelper {
    pub fn url(&self) -> &str {
        &self.url
    }
    pub fn get(&self, agent: &Agent) -> Result<Response, AgentError> {
        Ok(agent.get(self.url()).call()?)
    }
    pub fn with_enc2(mut self, enc2: &str) -> Self {
        self.url += "&enc2=";
        self.url += enc2;
        self
    }
    pub fn with_validate(mut self, validate: &str) -> Self {
        self.url += "&validate=";
        self.url += validate;
        self
    }
    pub fn path_enc_by_pre_sign_result_msg(self, msg: String) -> Self {
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

    fn deref(&self) -> &Self::Target {
        self.url()
    }
}
impl DerefMut for PPTSignHelper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.url
    }
}

impl From<String> for PPTSignHelper {
    fn from(s: String) -> Self {
        Self { url: s }
    }
}