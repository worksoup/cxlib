use onceinit::{OnceInit, OnceInitError, StaticDefault};
use std::fmt::{Display, Formatter};

pub enum Protocol {
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
impl Protocol {
    pub fn get(&self) -> String {
        PROTOCOL.get(self)
    }
    pub fn set(&self, value: &str) {
        PROTOCOL.set(self, value)
    }
    pub fn store() -> Result<(), cxsign_error::Error> {
        PROTOCOL.store()
    }
    pub fn update(&self, value: &str) -> bool {
        PROTOCOL.update(self, value)
    }
}
impl Display for Protocol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.get().fmt(f)
    }
}
pub trait ProtocolTrait<Protocol>: Sync {
    fn get(&self, t: &Protocol) -> String;

    fn set(&self, t: &Protocol, value: &str);
    fn store(&self) -> Result<(), cxsign_error::Error>;
    fn update(&self, t: &Protocol, value: &str) -> bool;
}

pub struct CXProtocol;

// 查询活动
static ACTIVE_LIST: &str = "https://mobilelearn.chaoxing.com/v2/apis/active/student/activelist";
static CAPTCHA_ID: &str = "Qt9FIw9o4pwRjOyqM6yizZBh682qN2TU";
// 获取滑块。
static GET_CAPTCHA: &str = "https://captcha.chaoxing.com/captcha/get/verification/image";
// 滑块验证。
static CHECK_CAPTCHA: &str = "https://captcha.chaoxing.com/captcha/check/verification/result";
// 获取服务器时间。
static GET_SERVER_TIME: &str = "https://captcha.chaoxing.com/captcha/get/conf";
static MY_SIGN_CAPTCHA_UTILS: &str =
    "https://mobilelearn.chaoxing.com/front/mobile/sign/js/mySignCaptchaUtils.js";
// 签到码检查
static CHECK_SIGNCODE: &str =
    "https://mobilelearn.chaoxing.com/widget/sign/pcStuSignController/checkSignCode";
// 签到信息获取
static SIGN_DETAIL: &str = "https://mobilelearn.chaoxing.com/newsign/signDetail";
// 登录页
static LOGIN_PAGE: &str =
    "https://passport2.chaoxing.com/mlogin?fid=&newversion=true&refer=http%3A%2F%2Fi.chaoxing.com";
// 非明文密码登录
static LOGIN_ENC: &str = "https://passport2.chaoxing.com/fanyalogin";
// 超星网盘页
static PAN_CHAOXING: &str = "https://pan-yz.chaoxing.com";
// 网盘列表
static PAN_LIST: &str = "https://pan-yz.chaoxing.com/opt/listres";
// 获取超星云盘的 token
static PAN_TOKEN: &str = "https://pan-yz.chaoxing.com/api/token/uservalid";
// 网盘上传接口
static PAN_UPLOAD: &str = "https://pan-yz.chaoxing.com/upload";
static QRCODE_PAT: &str = "https://mobilelearn.chaoxing.com/widget/sign/e";
// analysis
static ANALYSIS: &str = "https://mobilelearn.chaoxing.com/pptSign/analysis";
// analysis 2
static ANALYSIS2: &str = "https://mobilelearn.chaoxing.com/pptSign/analysis2";
// 获取签到之后的信息，例如签到时的 ip, UA, 时间等
// 参见 "http://mobilelearn.chaoxing.com/page/sign/signIn?courseId=$&classId=$&activeId=$&fid=$"
static GET_ATTEND_INFO: &str = "https://mobilelearn.chaoxing.com/v2/apis/sign/getAttendInfo";
// 签到
static PPT_SIGN: &str = "https://mobilelearn.chaoxing.com/pptSign/stuSignajax";
// 预签到
static PRE_SIGN: &str = "https://mobilelearn.chaoxing.com/newsign/preSign";
// 获取课程
static BACK_CLAZZ_DATA: &str = "https://mooc1-api.chaoxing.com/mycourse/backclazzdata";
// 获取位置信息列表
static GET_LOCATION_LOG: &str = "https://mobilelearn.chaoxing.com/v2/apis/sign/getLocationLog";
// 账号设置页
static ACCOUNT_MANAGE: &str = "https://passport2.chaoxing.com/mooc/accountManage";
static USER_AGENT: &str = "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36 com.chaoxing.mobile.xuezaixidian/ChaoXingStudy_1000149_5.3.1_android_phone_5000_83";

impl ProtocolTrait<Protocol> for CXProtocol {
    fn get(&self, t: &Protocol) -> String {
        match t {
            Protocol::ActiveList => ACTIVE_LIST,
            Protocol::GetCaptcha => GET_CAPTCHA,
            Protocol::CheckCaptcha => CHECK_CAPTCHA,
            Protocol::GetServerTime => GET_SERVER_TIME,
            Protocol::MySignCaptchaUtils => MY_SIGN_CAPTCHA_UTILS,
            Protocol::CheckSigncode => CHECK_SIGNCODE,
            Protocol::SignDetail => SIGN_DETAIL,
            Protocol::LoginPage => LOGIN_PAGE,
            Protocol::LoginEnc => LOGIN_ENC,
            Protocol::PanChaoxing => PAN_CHAOXING,
            Protocol::PanList => PAN_LIST,
            Protocol::PanToken => PAN_TOKEN,
            Protocol::PanUpload => PAN_UPLOAD,
            Protocol::Analysis => ANALYSIS,
            Protocol::Analysis2 => ANALYSIS2,
            Protocol::GetAttendInfo => GET_ATTEND_INFO,
            Protocol::PptSign => PPT_SIGN,
            Protocol::PreSign => PRE_SIGN,
            Protocol::BackClazzData => BACK_CLAZZ_DATA,
            Protocol::GetLocationLog => GET_LOCATION_LOG,
            Protocol::AccountManage => ACCOUNT_MANAGE,
            Protocol::CaptchaId => CAPTCHA_ID,
            Protocol::UserAgent => USER_AGENT,
            Protocol::QrcodePat => QRCODE_PAT,
        }
        .to_owned()
    }

    fn set(&self, _: &Protocol, _: &str) {}

    fn store(&self) -> Result<(), cxsign_error::Error> {
        Err(cxsign_error::Error::FunctionIsDisabled(
            "默认实现不支持此操作。".to_string(),
        ))
    }

    fn update(&self, _: &Protocol, _: &str) -> bool {
        false
    }
}

impl StaticDefault for dyn ProtocolTrait<Protocol> {
    fn static_default() -> &'static Self {
        static DEFAULT: CXProtocol = CXProtocol;
        &DEFAULT
    }
}

static PROTOCOL: OnceInit<dyn ProtocolTrait<Protocol>> = OnceInit::new();

pub fn set_protocol(protocol: &'static impl ProtocolTrait<Protocol>) -> Result<(), OnceInitError> {
    PROTOCOL.set_data(protocol)
}

pub fn set_boxed_protocol(
    protocol: Box<impl ProtocolTrait<Protocol> + 'static>,
) -> Result<(), OnceInitError> {
    PROTOCOL.set_boxed_data(protocol)
}
