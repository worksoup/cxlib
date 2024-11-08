use cxsign_protocol::{CXProtocol, ProtocolEnum, ProtocolTrait};
use log::{error, warn};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{ErrorKind, Read, Write},
};

#[derive(Serialize, Deserialize)]
struct ProtocolData {
    active_list: Option<String>,
    get_captcha: Option<String>,
    check_captcha: Option<String>,
    get_server_time: Option<String>,
    my_sign_captcha_utils: Option<String>,
    check_signcode: Option<String>,
    sign_detail: Option<String>,
    login_page: Option<String>,
    login_enc: Option<String>,
    pan_chaoxing: Option<String>,
    pan_list: Option<String>,
    pan_token: Option<String>,
    pan_upload: Option<String>,
    analysis: Option<String>,
    analysis2: Option<String>,
    get_attend_info: Option<String>,
    ppt_sign: Option<String>,
    pre_sign: Option<String>,
    back_clazz_data: Option<String>,
    get_location_log: Option<String>,
    account_manage: Option<String>,
    captcha_id: Option<String>,
    user_agent: Option<String>,
    qrcode_pat: Option<String>,
}
impl ProtocolData {
    fn map_by_enum<'a, T>(
        &'a self,
        t: &ProtocolEnum,
        do_something: impl Fn(&'a Option<String>) -> T,
    ) -> T {
        match t {
            ProtocolEnum::ActiveList => do_something(&self.active_list),
            ProtocolEnum::GetCaptcha => do_something(&self.get_captcha),
            ProtocolEnum::CheckCaptcha => do_something(&self.check_captcha),
            ProtocolEnum::GetServerTime => do_something(&self.get_server_time),
            ProtocolEnum::MySignCaptchaUtils => do_something(&self.my_sign_captcha_utils),
            ProtocolEnum::CheckSigncode => do_something(&self.check_signcode),
            ProtocolEnum::SignDetail => do_something(&self.sign_detail),
            ProtocolEnum::LoginPage => do_something(&self.login_page),
            ProtocolEnum::LoginEnc => do_something(&self.login_enc),
            ProtocolEnum::PanChaoxing => do_something(&self.pan_chaoxing),
            ProtocolEnum::PanList => do_something(&self.pan_list),
            ProtocolEnum::PanToken => do_something(&self.pan_token),
            ProtocolEnum::PanUpload => do_something(&self.pan_upload),
            ProtocolEnum::Analysis => do_something(&self.analysis),
            ProtocolEnum::Analysis2 => do_something(&self.analysis2),
            ProtocolEnum::GetAttendInfo => do_something(&self.get_attend_info),
            ProtocolEnum::PptSign => do_something(&self.ppt_sign),
            ProtocolEnum::PreSign => do_something(&self.pre_sign),
            ProtocolEnum::BackClazzData => do_something(&self.back_clazz_data),
            ProtocolEnum::GetLocationLog => do_something(&self.get_location_log),
            ProtocolEnum::AccountManage => do_something(&self.account_manage),
            ProtocolEnum::CaptchaId => do_something(&self.captcha_id),
            ProtocolEnum::UserAgent => do_something(&self.user_agent),
            ProtocolEnum::QrcodePat => do_something(&self.qrcode_pat),
        }
    }
    fn map_by_enum_mut<'a, T>(
        &'a mut self,
        t: &ProtocolEnum,
        do_something: impl Fn(&'a mut Option<String>) -> T,
    ) -> T {
        match t {
            ProtocolEnum::ActiveList => do_something(&mut self.active_list),
            ProtocolEnum::GetCaptcha => do_something(&mut self.get_captcha),
            ProtocolEnum::CheckCaptcha => do_something(&mut self.check_captcha),
            ProtocolEnum::GetServerTime => do_something(&mut self.get_server_time),
            ProtocolEnum::MySignCaptchaUtils => do_something(&mut self.my_sign_captcha_utils),
            ProtocolEnum::CheckSigncode => do_something(&mut self.check_signcode),
            ProtocolEnum::SignDetail => do_something(&mut self.sign_detail),
            ProtocolEnum::LoginPage => do_something(&mut self.login_page),
            ProtocolEnum::LoginEnc => do_something(&mut self.login_enc),
            ProtocolEnum::PanChaoxing => do_something(&mut self.pan_chaoxing),
            ProtocolEnum::PanList => do_something(&mut self.pan_list),
            ProtocolEnum::PanToken => do_something(&mut self.pan_token),
            ProtocolEnum::PanUpload => do_something(&mut self.pan_upload),
            ProtocolEnum::Analysis => do_something(&mut self.analysis),
            ProtocolEnum::Analysis2 => do_something(&mut self.analysis2),
            ProtocolEnum::GetAttendInfo => do_something(&mut self.get_attend_info),
            ProtocolEnum::PptSign => do_something(&mut self.ppt_sign),
            ProtocolEnum::PreSign => do_something(&mut self.pre_sign),
            ProtocolEnum::BackClazzData => do_something(&mut self.back_clazz_data),
            ProtocolEnum::GetLocationLog => do_something(&mut self.get_location_log),
            ProtocolEnum::AccountManage => do_something(&mut self.account_manage),
            ProtocolEnum::CaptchaId => do_something(&mut self.captcha_id),
            ProtocolEnum::UserAgent => do_something(&mut self.user_agent),
            ProtocolEnum::QrcodePat => do_something(&mut self.qrcode_pat),
        }
    }
    fn set(&mut self, t: &ProtocolEnum, value: &str) {
        self.map_by_enum_mut(t, |t| t.replace(value.to_owned()));
    }
    fn update(&mut self, t: &ProtocolEnum, value: &str) -> bool {
        self.map_by_enum_mut(t, |t| {
            let not_to_update = t.as_ref().is_some_and(|v| v == value);
            t.replace(value.to_owned());
            !not_to_update
        })
    }
}
impl Default for ProtocolData {
    fn default() -> Self {
        fn get(t: ProtocolEnum) -> Option<String> {
            Some(CXProtocol.get(&t).to_owned())
        }
        Self {
            active_list: get(ProtocolEnum::ActiveList),
            get_captcha: get(ProtocolEnum::GetCaptcha),
            check_captcha: get(ProtocolEnum::CheckCaptcha),
            get_server_time: get(ProtocolEnum::GetServerTime),
            my_sign_captcha_utils: get(ProtocolEnum::MySignCaptchaUtils),
            check_signcode: get(ProtocolEnum::CheckSigncode),
            sign_detail: get(ProtocolEnum::SignDetail),
            login_page: get(ProtocolEnum::LoginPage),
            login_enc: get(ProtocolEnum::LoginEnc),
            pan_chaoxing: get(ProtocolEnum::PanChaoxing),
            pan_list: get(ProtocolEnum::PanList),
            pan_token: get(ProtocolEnum::PanToken),
            pan_upload: get(ProtocolEnum::PanUpload),
            analysis: get(ProtocolEnum::Analysis),
            analysis2: get(ProtocolEnum::Analysis2),
            get_attend_info: get(ProtocolEnum::GetAttendInfo),
            ppt_sign: get(ProtocolEnum::PptSign),
            pre_sign: get(ProtocolEnum::PreSign),
            back_clazz_data: get(ProtocolEnum::BackClazzData),
            get_location_log: get(ProtocolEnum::GetLocationLog),
            account_manage: get(ProtocolEnum::AccountManage),
            captcha_id: get(ProtocolEnum::CaptchaId),
            user_agent: get(ProtocolEnum::UserAgent),
            qrcode_pat: get(ProtocolEnum::QrcodePat),
        }
    }
}

pub struct DefaultCXProtocol {
    data: ProtocolData,
    file: File,
}
impl DefaultCXProtocol {
    pub fn init() -> Result<(), cxsign_error::Error> {
        let protocol_config_path = cxsign_dir::Dir::get_config_file_path("protocol.toml");
        let mut file = match File::open(&protocol_config_path) {
            Ok(file) => file,
            Err(e) => match e.kind() {
                ErrorKind::NotFound => {
                    warn!("配置文件 `protocol.toml` 不存在，将新建。");
                    File::create(&protocol_config_path)?
                }
                _ => {
                    error!("无法打开配置文件 `protocol.toml`: {}.", e.to_string());
                    return Err(e.into());
                }
            },
        };
        let mut s = String::new();
        let _ = file.read_to_string(&mut s)?;
        let data: ProtocolData =
            toml::from_str(&s).map_err(|e| cxsign_error::Error::ParseError(e.to_string()))?;
        let protocol = DefaultCXProtocol { data, file };
        cxsign_protocol::set_boxed_protocol(Box::new(protocol))
            .map_err(|_| cxsign_error::Error::SetProtocolError)
    }
    pub fn map_by_enum<'a, T: 'a>(
        &'a self,
        t: &ProtocolEnum,
        do_something: impl Fn(&'a Option<String>) -> T,
    ) -> T {
        self.data.map_by_enum(t, do_something)
    }
    pub fn map_by_enum_mut<'a, T: 'a>(
        &'a mut self,
        t: &ProtocolEnum,
        do_something: impl Fn(&'a mut Option<String>) -> T,
    ) -> T {
        self.data.map_by_enum_mut(t, do_something)
    }
    pub fn set(&mut self, t: &ProtocolEnum, value: &str) {
        self.data.set(t, value)
    }
    pub fn store(&mut self) -> Result<(), cxsign_error::Error> {
        let toml = toml::to_string_pretty(&self.data)
            .map_err(|e| cxsign_error::Error::ParseError(e.to_string()))?;
        Ok(self.file.write_all(toml.as_bytes())?)
    }
    /// 更新字段，相当于 [`set`](Self::set) + [`store`](Self::store), 具体逻辑为：若传入值与原有值不同，则更新字段并保存至文件。保存成功返回 `true`, 其余情况返回 `false`.
    pub fn update(&mut self, t: &ProtocolEnum, value: &str) -> bool {
        self.data.update(t, value) && self.store().is_ok()
    }
}
impl ProtocolTrait for DefaultCXProtocol {
    fn get(&self, t: &ProtocolEnum) -> &str {
        if let Some(r) = self.data.map_by_enum(t, |a| a.as_ref().map(|a| a.as_str())) {
            r
        } else {
            CXProtocol.get(t)
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::cx_protocol::ProtocolData;

    #[test]
    fn test_default() {
        let content = toml::to_string_pretty(&ProtocolData::default()).unwrap();
        println!("{content}");
    }
}