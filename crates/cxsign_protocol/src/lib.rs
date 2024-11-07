use onceinit::{OnceInit, OnceInitError, StaticDefault};
use std::fmt::{Display, Formatter};
use std::ops::Deref;

pub enum ProtocolEnum {
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
impl Deref for ProtocolEnum {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        get(self)
    }
}
impl Display for ProtocolEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.deref().fmt(f)
    }
}
pub trait ProtocolTrait: Sync {
    fn get(&self, t: &ProtocolEnum) -> &str;
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

impl ProtocolTrait for CXProtocol {
    fn get(&self, t: &ProtocolEnum) -> &str {
        match t {
            ProtocolEnum::ActiveList => ACTIVE_LIST,
            ProtocolEnum::GetCaptcha => GET_CAPTCHA,
            ProtocolEnum::CheckCaptcha => CHECK_CAPTCHA,
            ProtocolEnum::GetServerTime => GET_SERVER_TIME,
            ProtocolEnum::MySignCaptchaUtils => MY_SIGN_CAPTCHA_UTILS,
            ProtocolEnum::CheckSigncode => CHECK_SIGNCODE,
            ProtocolEnum::SignDetail => SIGN_DETAIL,
            ProtocolEnum::LoginPage => LOGIN_PAGE,
            ProtocolEnum::LoginEnc => LOGIN_ENC,
            ProtocolEnum::PanChaoxing => PAN_CHAOXING,
            ProtocolEnum::PanList => PAN_LIST,
            ProtocolEnum::PanToken => PAN_TOKEN,
            ProtocolEnum::PanUpload => PAN_UPLOAD,
            ProtocolEnum::Analysis => ANALYSIS,
            ProtocolEnum::Analysis2 => ANALYSIS2,
            ProtocolEnum::GetAttendInfo => GET_ATTEND_INFO,
            ProtocolEnum::PptSign => PPT_SIGN,
            ProtocolEnum::PreSign => PRE_SIGN,
            ProtocolEnum::BackClazzData => BACK_CLAZZ_DATA,
            ProtocolEnum::GetLocationLog => GET_LOCATION_LOG,
            ProtocolEnum::AccountManage => ACCOUNT_MANAGE,
            ProtocolEnum::CaptchaId => CAPTCHA_ID,
            ProtocolEnum::UserAgent => USER_AGENT,
            ProtocolEnum::QrcodePat => QRCODE_PAT,
        }
    }
}

impl StaticDefault for dyn ProtocolTrait {
    fn static_default() -> &'static Self {
        static DEFAULT: CXProtocol = CXProtocol;
        &DEFAULT
    }
}

static PROTOCOL: OnceInit<dyn ProtocolTrait> = OnceInit::new();

pub fn set_protocol(protocol: &'static impl ProtocolTrait) -> Result<(), OnceInitError> {
    PROTOCOL.set_data(protocol)
}

pub fn set_boxed_protocol(
    protocol: Box<impl ProtocolTrait + 'static>,
) -> Result<(), OnceInitError> {
    PROTOCOL.set_boxed_data(protocol)
}

pub fn get(t: &ProtocolEnum) -> &'static str {
    PROTOCOL.get(t)
}
