use crate::{ProtocolDataTrait, ProtocolItemTrait, ProtocolTrait, PROTOCOL};
use log::warn;
use onceinit::OnceInit;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter},
    fs::{File, OpenOptions},
    io::{ErrorKind, Read, Write},
    ops::Deref,
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
};

pub enum ProtocolItem {
    ActiveList,
    GetCaptcha,
    CheckCaptcha,
    GetServerTime,
    MySignCaptchaUtils,
    CheckSigncode,
    SignDetail,
    LoginPage,
    LoginEnc,
    PanChaoxing,
    PanList,
    PanToken,
    PanUpload,
    Analysis,
    Analysis2,
    GetAttendInfo,
    PptSign,
    PreSign,
    BackClazzData,
    GetLocationLog,
    AccountManage,
    CaptchaId,
    UserAgent,
    QrcodePat,
}
impl ProtocolItem {
    // 查询活动
    pub const ACTIVE_LIST: &'static str =
        "https://mobilelearn.chaoxing.com/v2/apis/active/student/activelist";
    pub const CAPTCHA_ID: &'static str = "Qt9FIw9o4pwRjOyqM6yizZBh682qN2TU";
    // 获取滑块。
    pub const GET_CAPTCHA: &'static str =
        "https://captcha.chaoxing.com/captcha/get/verification/image";
    // 滑块验证。
    pub const CHECK_CAPTCHA: &'static str =
        "https://captcha.chaoxing.com/captcha/check/verification/result";
    // 获取服务器时间。
    pub const GET_SERVER_TIME: &'static str = "https://captcha.chaoxing.com/captcha/get/conf";
    pub const MY_SIGN_CAPTCHA_UTILS: &'static str =
        "https://mobilelearn.chaoxing.com/front/mobile/sign/js/mySignCaptchaUtils.js";
    // 签到码检查
    pub const CHECK_SIGNCODE: &'static str =
        "https://mobilelearn.chaoxing.com/widget/sign/pcStuSignController/checkSignCode";
    // 签到信息获取
    pub const SIGN_DETAIL: &'static str = "https://mobilelearn.chaoxing.com/newsign/signDetail";
    // 登录页
    pub const LOGIN_PAGE: &'static str =
        "https://passport2.chaoxing.com/mlogin?fid=&newversion=true&refer=http%3A%2F%2Fi.chaoxing.com";
    // 非明文密码登录
    pub const LOGIN_ENC: &'static str = "https://passport2.chaoxing.com/fanyalogin";
    // 超星网盘页
    pub const PAN_CHAOXING: &'static str = "https://pan-yz.chaoxing.com";
    // 网盘列表
    pub const PAN_LIST: &'static str = "https://pan-yz.chaoxing.com/opt/listres";
    // 获取超星云盘的 token
    pub const PAN_TOKEN: &'static str = "https://pan-yz.chaoxing.com/api/token/uservalid";
    // 网盘上传接口
    pub const PAN_UPLOAD: &'static str = "https://pan-yz.chaoxing.com/upload";
    pub const QRCODE_PAT: &'static str = "https://mobilelearn.chaoxing.com/widget/sign/e";
    // analysis
    pub const ANALYSIS: &'static str = "https://mobilelearn.chaoxing.com/pptSign/analysis";
    // analysis 2
    pub const ANALYSIS2: &'static str = "https://mobilelearn.chaoxing.com/pptSign/analysis2";
    // 获取签到之后的信息，例如签到时的 ip, UA, 时间等
    // 参见 "http://mobilelearn.chaoxing.com/page/sign/signIn?courseId=$&classId=$&activeId=$&fid=$"
    pub const GET_ATTEND_INFO: &'static str =
        "https://mobilelearn.chaoxing.com/v2/apis/sign/getAttendInfo";
    // 签到
    pub const PPT_SIGN: &'static str = "https://mobilelearn.chaoxing.com/pptSign/stuSignajax";
    // 预签到
    pub const PRE_SIGN: &'static str = "https://mobilelearn.chaoxing.com/newsign/preSign";
    // 获取课程
    pub const BACK_CLAZZ_DATA: &'static str =
        "https://mooc1-api.chaoxing.com/mycourse/backclazzdata";
    // 获取位置信息列表
    pub const GET_LOCATION_LOG: &'static str =
        "https://mobilelearn.chaoxing.com/v2/apis/sign/getLocationLog";
    // 账号设置页
    pub const ACCOUNT_MANAGE: &'static str = "https://passport2.chaoxing.com/mooc/accountManage";
    pub const USER_AGENT: &'static str = "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36 com.chaoxing.mobile.xuezaixidian/ChaoXingStudy_1000149_5.3.1_android_phone_5000_83";
}
impl ProtocolItemTrait for ProtocolItem {
    type ProtocolData = ProtocolData;

    fn config_file_name() -> &'static str {
        "protocol.toml"
    }

    fn get_protocol_() -> &'static OnceInit<dyn ProtocolTrait<Self>> {
        &PROTOCOL
    }
    fn get_protocol() -> &'static (dyn ProtocolTrait<ProtocolItem> + 'static) {
        PROTOCOL.deref()
    }

    fn get_default(&self) -> String {
        match self {
            Self::ActiveList => Self::ACTIVE_LIST.to_string(),
            Self::GetCaptcha => Self::GET_CAPTCHA.to_string(),
            Self::CheckCaptcha => Self::CHECK_CAPTCHA.to_string(),
            Self::GetServerTime => Self::GET_SERVER_TIME.to_string(),
            Self::MySignCaptchaUtils => Self::MY_SIGN_CAPTCHA_UTILS.to_string(),
            Self::CheckSigncode => Self::CHECK_SIGNCODE.to_string(),
            Self::SignDetail => Self::SIGN_DETAIL.to_string(),
            Self::LoginPage => Self::LOGIN_PAGE.to_string(),
            Self::LoginEnc => Self::LOGIN_ENC.to_string(),
            Self::PanChaoxing => Self::PAN_CHAOXING.to_string(),
            Self::PanList => Self::PAN_LIST.to_string(),
            Self::PanToken => Self::PAN_TOKEN.to_string(),
            Self::PanUpload => Self::PAN_UPLOAD.to_string(),
            Self::Analysis => Self::ANALYSIS.to_string(),
            Self::Analysis2 => Self::ANALYSIS2.to_string(),
            Self::GetAttendInfo => Self::GET_ATTEND_INFO.to_string(),
            Self::PptSign => Self::PPT_SIGN.to_string(),
            Self::PreSign => Self::PRE_SIGN.to_string(),
            Self::BackClazzData => Self::BACK_CLAZZ_DATA.to_string(),
            Self::GetLocationLog => Self::GET_LOCATION_LOG.to_string(),
            Self::AccountManage => Self::ACCOUNT_MANAGE.to_string(),
            Self::CaptchaId => Self::CAPTCHA_ID.to_string(),
            Self::UserAgent => Self::USER_AGENT.to_string(),
            Self::QrcodePat => Self::QRCODE_PAT.to_string(),
        }
    }
}
impl Display for ProtocolItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.get().fmt(f)
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ProtocolData {
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
impl ProtocolDataTrait for ProtocolData {
    type ProtocolItem = ProtocolItem;

    fn map_by_enum<'a, T>(
        &'a self,
        t: &ProtocolItem,
        do_something: impl Fn(&'a Option<String>) -> T,
    ) -> T {
        match t {
            ProtocolItem::ActiveList => do_something(&self.active_list),
            ProtocolItem::GetCaptcha => do_something(&self.get_captcha),
            ProtocolItem::CheckCaptcha => do_something(&self.check_captcha),
            ProtocolItem::GetServerTime => do_something(&self.get_server_time),
            ProtocolItem::MySignCaptchaUtils => do_something(&self.my_sign_captcha_utils),
            ProtocolItem::CheckSigncode => do_something(&self.check_signcode),
            ProtocolItem::SignDetail => do_something(&self.sign_detail),
            ProtocolItem::LoginPage => do_something(&self.login_page),
            ProtocolItem::LoginEnc => do_something(&self.login_enc),
            ProtocolItem::PanChaoxing => do_something(&self.pan_chaoxing),
            ProtocolItem::PanList => do_something(&self.pan_list),
            ProtocolItem::PanToken => do_something(&self.pan_token),
            ProtocolItem::PanUpload => do_something(&self.pan_upload),
            ProtocolItem::Analysis => do_something(&self.analysis),
            ProtocolItem::Analysis2 => do_something(&self.analysis2),
            ProtocolItem::GetAttendInfo => do_something(&self.get_attend_info),
            ProtocolItem::PptSign => do_something(&self.ppt_sign),
            ProtocolItem::PreSign => do_something(&self.pre_sign),
            ProtocolItem::BackClazzData => do_something(&self.back_clazz_data),
            ProtocolItem::GetLocationLog => do_something(&self.get_location_log),
            ProtocolItem::AccountManage => do_something(&self.account_manage),
            ProtocolItem::CaptchaId => do_something(&self.captcha_id),
            ProtocolItem::UserAgent => do_something(&self.user_agent),
            ProtocolItem::QrcodePat => do_something(&self.qrcode_pat),
        }
    }
    fn map_by_enum_mut<'a, T>(
        &'a mut self,
        t: &ProtocolItem,
        do_something: impl Fn(&'a mut Option<String>) -> T,
    ) -> T {
        match t {
            ProtocolItem::ActiveList => do_something(&mut self.active_list),
            ProtocolItem::GetCaptcha => do_something(&mut self.get_captcha),
            ProtocolItem::CheckCaptcha => do_something(&mut self.check_captcha),
            ProtocolItem::GetServerTime => do_something(&mut self.get_server_time),
            ProtocolItem::MySignCaptchaUtils => do_something(&mut self.my_sign_captcha_utils),
            ProtocolItem::CheckSigncode => do_something(&mut self.check_signcode),
            ProtocolItem::SignDetail => do_something(&mut self.sign_detail),
            ProtocolItem::LoginPage => do_something(&mut self.login_page),
            ProtocolItem::LoginEnc => do_something(&mut self.login_enc),
            ProtocolItem::PanChaoxing => do_something(&mut self.pan_chaoxing),
            ProtocolItem::PanList => do_something(&mut self.pan_list),
            ProtocolItem::PanToken => do_something(&mut self.pan_token),
            ProtocolItem::PanUpload => do_something(&mut self.pan_upload),
            ProtocolItem::Analysis => do_something(&mut self.analysis),
            ProtocolItem::Analysis2 => do_something(&mut self.analysis2),
            ProtocolItem::GetAttendInfo => do_something(&mut self.get_attend_info),
            ProtocolItem::PptSign => do_something(&mut self.ppt_sign),
            ProtocolItem::PreSign => do_something(&mut self.pre_sign),
            ProtocolItem::BackClazzData => do_something(&mut self.back_clazz_data),
            ProtocolItem::GetLocationLog => do_something(&mut self.get_location_log),
            ProtocolItem::AccountManage => do_something(&mut self.account_manage),
            ProtocolItem::CaptchaId => do_something(&mut self.captcha_id),
            ProtocolItem::UserAgent => do_something(&mut self.user_agent),
            ProtocolItem::QrcodePat => do_something(&mut self.qrcode_pat),
        }
    }
}
impl Default for ProtocolData {
    fn default() -> Self {
        Self {
            active_list: Some(ProtocolItem::ACTIVE_LIST.to_string()),
            get_captcha: Some(ProtocolItem::GET_CAPTCHA.to_string()),
            check_captcha: Some(ProtocolItem::CHECK_CAPTCHA.to_string()),
            get_server_time: Some(ProtocolItem::GET_SERVER_TIME.to_string()),
            my_sign_captcha_utils: Some(ProtocolItem::MY_SIGN_CAPTCHA_UTILS.to_string()),
            check_signcode: Some(ProtocolItem::CHECK_SIGNCODE.to_string()),
            sign_detail: Some(ProtocolItem::SIGN_DETAIL.to_string()),
            login_page: Some(ProtocolItem::LOGIN_PAGE.to_string()),
            login_enc: Some(ProtocolItem::LOGIN_ENC.to_string()),
            pan_chaoxing: Some(ProtocolItem::PAN_CHAOXING.to_string()),
            pan_list: Some(ProtocolItem::PAN_LIST.to_string()),
            pan_token: Some(ProtocolItem::PAN_TOKEN.to_string()),
            pan_upload: Some(ProtocolItem::PAN_UPLOAD.to_string()),
            analysis: Some(ProtocolItem::ANALYSIS.to_string()),
            analysis2: Some(ProtocolItem::ANALYSIS2.to_string()),
            get_attend_info: Some(ProtocolItem::GET_ATTEND_INFO.to_string()),
            ppt_sign: Some(ProtocolItem::PPT_SIGN.to_string()),
            pre_sign: Some(ProtocolItem::PRE_SIGN.to_string()),
            back_clazz_data: Some(ProtocolItem::BACK_CLAZZ_DATA.to_string()),
            get_location_log: Some(ProtocolItem::GET_LOCATION_LOG.to_string()),
            account_manage: Some(ProtocolItem::ACCOUNT_MANAGE.to_string()),
            captcha_id: Some(ProtocolItem::CAPTCHA_ID.to_string()),
            user_agent: Some(ProtocolItem::USER_AGENT.to_string()),
            qrcode_pat: Some(ProtocolItem::QRCODE_PAT.to_string()),
        }
    }
}

pub struct CXProtocol<ProtocolData> {
    data: Arc<RwLock<ProtocolData>>,
    file: Option<Arc<Mutex<File>>>,
}
impl<Protocol, ProtocolData> CXProtocol<ProtocolData>
where
    ProtocolData: Default
        + for<'de> serde::Deserialize<'de>
        + serde::Serialize
        + Send
        + Sync
        + 'static
        + ProtocolDataTrait<ProtocolItem = Protocol>,
{
    /// # init
    /// 读取配置文件 `protocol.toml` 并构造 `ProtocolData`.
    /// 若文件不存在，则尝试新建并使用默认值。
    ///
    /// 接着将协议设置为 `DefaultCXProtocol`.
    ///
    /// # Errors
    ///
    /// 在设置协议出错时返回 [`SetProtocolError`](cxlib_error::Error::ParseError).
    pub fn load(protocol_config_path: &PathBuf) -> Result<Self, cxlib_error::Error> {
        let metadata = protocol_config_path.metadata();
        let mut read = false;
        let mut file = match metadata {
            Ok(metadata) => {
                if metadata.is_file() {
                    OpenOptions::new()
                        .read(true)
                        .write(true)
                        .open(protocol_config_path.as_path())
                        .ok()
                } else {
                    warn!("文件位置被目录占用。");
                    None
                }
            }
            Err(e) => match e.kind() {
                ErrorKind::NotFound => {
                    warn!("配置文件 `protocol.toml` 不存在，将新建。");
                    read = true;
                    File::create(protocol_config_path).ok()
                }
                _ => {
                    warn!("无法打开配置文件 `protocol.toml`: {}.", e.to_string());
                    None
                }
            },
        };
        let data = file
            .as_mut()
            .map(|f| {
                if read {
                    let data = ProtocolData::default();
                    let config = toml::to_string_pretty(&data).unwrap();
                    let _ = f.write_all(config.as_bytes());
                    data
                } else {
                    let mut config = String::new();
                    f.read_to_string(&mut config)
                        .ok()
                        .and_then(|_| toml::from_str(&config).ok())
                        .unwrap_or_default()
                }
            })
            .unwrap_or_default();
        let data = Arc::new(RwLock::new(data));
        let file = file.map(|f| Arc::new(Mutex::new(f)));
        Ok(CXProtocol { data, file })
    }
}
impl<ProtocolItem, ProtocolData> CXProtocol<ProtocolData>
where
    ProtocolItem: ProtocolItemTrait,
    ProtocolData: Default
        + for<'de> serde::Deserialize<'de>
        + serde::Serialize
        + Send
        + Sync
        + 'static
        + ProtocolDataTrait<ProtocolItem = ProtocolItem>,
{
    /// # init
    /// 读取配置文件 `protocol.toml` 并构造 `ProtocolData`.
    /// 若文件不存在，则尝试新建并使用默认值。
    ///
    /// 接着将协议设置为 `DefaultCXProtocol`.
    ///
    /// # Errors
    ///
    /// 在设置协议出错时返回 [`SetProtocolError`](cxlib_error::Error::SetProtocolError).
    pub fn init() -> Result<(), cxlib_error::Error> {
        let protocol_config_path =
            cxlib_store::Dir::get_config_file_path(ProtocolItem::config_file_name());
        let protocol = CXProtocol::<ProtocolData>::load(&protocol_config_path)?;
        ProtocolItem::set_boxed_protocol(Box::new(protocol))
            .map_err(|_| cxlib_error::Error::SetProtocolError)
    }
}
impl<ProtocolItem, ProtocolData> ProtocolTrait<ProtocolItem> for CXProtocol<ProtocolData>
where
    ProtocolItem: ProtocolItemTrait,
    ProtocolData: Default
        + for<'de> serde::Deserialize<'de>
        + serde::Serialize
        + Send
        + Sync
        + ProtocolDataTrait<ProtocolItem = ProtocolItem>,
{
    fn get(&self, t: &ProtocolItem) -> String {
        if let Some(r) = self
            .data
            .read()
            .unwrap()
            .map_by_enum(t, |a| a.as_ref().map(|a| a.as_str()))
        {
            r.to_owned()
        } else {
            ProtocolItem::get_default(t)
        }
    }
    fn set(&self, t: &ProtocolItem, value: &str) {
        self.data.write().unwrap().set(t, value)
    }
    fn store(&self) -> Result<(), cxlib_error::Error> {
        let toml =
            toml::to_string_pretty(&*self.data.read().unwrap()).expect("若看到此消息说明有 bug.");
        self.file
            .as_ref()
            .map(|f| f.lock().unwrap().write_all(toml.as_bytes()))
            .transpose()?
            .ok_or_else(|| {
                cxlib_error::Error::FunctionIsDisabled(
                    "文件为只读状态，保存功能已禁用。".to_string(),
                )
            })
    }
    /// 更新字段，相当于 [`set`](Self::set) + [`store`](Self::store), 具体逻辑为：若传入值与原有值不同，则更新字段并保存至文件。保存成功返回 `true`, 其余情况返回 `false`.
    fn update(&self, t: &ProtocolItem, value: &str) -> bool {
        self.data.write().unwrap().update(t, value) && self.store().is_ok()
    }
}
#[cfg(test)]
mod tests {
    use crate::default_impl::ProtocolData;

    #[test]
    fn test_default() {
        let content = toml::to_string_pretty(&ProtocolData::default()).unwrap();
        println!("{content}");
        let null: ProtocolData = toml::from_str("").unwrap();
        println!("{null:?}");
        let content = toml::to_string_pretty(&null).unwrap();
        println!("{content:?}");
    }
}
