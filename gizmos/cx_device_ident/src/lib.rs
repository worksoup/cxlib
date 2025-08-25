use std::str::FromStr;

mod default_impl;

pub use default_impl::*;

pub trait DeviceCodeGen {
    fn user_agent_gen(&self) -> String;
    fn get_device_code(&self) -> String;
    #[inline(always)]
    fn parse_ua(s: &str) -> Result<Self, <Self as FromStr>::Err>
    where
        Self: FromStr,
    {
        s.parse()
    }
}
impl DeviceCodeGen for DefaultDeviceCodeGen {
    #[inline]
    fn user_agent_gen(&self) -> String {
        Self::user_agent_gen(
            self.base_ua(),
            self.custom_ident(),
            self.schild().as_deref(),
            self.device(),
            self.device_identifier(),
        )
    }

    #[inline]
    fn get_device_code(&self) -> String {
        Self::device_code_gen(self.device_identifier())
    }
}

#[derive(Debug)]
pub struct SimpleDeviceCodeGen {
    pub user_agent: String,
    pub device_code: String,
}
impl DeviceCodeGen for SimpleDeviceCodeGen {
    #[inline(always)]
    fn user_agent_gen(&self) -> String {
        self.user_agent.clone()
    }
    #[inline(always)]
    fn get_device_code(&self) -> String {
        self.device_code.clone()
    }
}
