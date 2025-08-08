use cxlib_error::AgentError;
use cxlib_error_utils::CxlibResultUtils;
use log::debug;
use std::{
    borrow::{Borrow, Cow},
    ffi::{OsStr, OsString},
    fmt::{Debug, Display},
    path::Path,
};
use ureq::{Agent, SendBody};

mod types_ {
    use cxlib_base_types::{SignDetail, UnhandledGeoAddrWithRange};
    use cxlib_error::AgentError;
    use getset2::Getset2;
    use serde::Deserialize;
    use ureq::Agent;

    use crate::collect::TypesProtocolTrait;

    /// 对于已经结束的课程，该字段将为空字符串。
    /// 也许有可能为 null. TODO: 后续需验证。
    #[derive(Deserialize, Clone)]
    #[serde(untagged)]
    pub enum StartTimeMills {
        Some(u64),
        None(String),
    }
    impl StartTimeMills {
        #[inline]
        pub fn some(&self) -> Option<u64> {
            match self {
                Self::Some(mills) => Some(*mills),
                Self::None(_) => None,
            }
        }
    }
    /// # ActivityRaw
    ///
    /// 未分类的活动类型，仅用于内部反序列化。
    ///
    /// 请参考 [`protocol::active_list`] 的响应数据。
    #[derive(Deserialize, Clone)]
    pub struct ActivityRaw {
        #[serde(rename = "nameOne")]
        pub name_one: String,
        pub id: i64,
        #[serde(rename = "otherId")]
        pub other_id: Option<String>,
        pub status: i32,
        #[serde(rename = "startTime")]
        pub start_time_mills: StartTimeMills,
    }
    /// 内部类型，用于反序列化。
    ///
    /// 请参考 [`protocol::active_list`] 的响应数据。
    #[derive(Deserialize, Getset2)]
    #[getset2(get_ref(pub))]
    pub struct Data {
        #[serde(rename = "activeList")]
        active_list: Vec<ActivityRaw>,
    }
    /// 内部类型，用于反序列化。
    ///
    /// 请参考 [`protocol::active_list`] 的响应数据。
    #[derive(Deserialize, Getset2)]
    #[getset2(get_ref(pub))]
    pub struct ActivityListRaw {
        data: Option<Data>,
    }
    #[derive(Debug, Clone, Deserialize, Getset2)]
    #[getset2(get_ref(pub))]
    pub struct LocationWithRangeAndActiveId {
        #[serde(rename = "activeid")]
        active_id: i64,
        #[serde(rename = "address")]
        addr: String,
        #[serde(rename = "longitude")]
        lon: f64,
        #[serde(rename = "latitude")]
        lat: f64,
        #[serde(rename = "locationrange")]
        range: String,
    }
    impl LocationWithRangeAndActiveId {
        #[inline]
        pub fn to_location_with_range(&self) -> UnhandledGeoAddrWithRange {
            UnhandledGeoAddrWithRange::new(
                self.addr.to_owned(),
                self.lon.to_string(),
                self.lat.to_string(),
                self.range.trim().parse().unwrap_or(100),
            )
        }
    }
    /// 课程签到所使用过的位置列表。
    #[derive(Debug, Clone, Deserialize, Getset2)]
    #[getset2(get_ref(pub))]
    pub struct LocationLogData {
        #[serde(rename = "data")]
        data: Vec<LocationWithRangeAndActiveId>,
    }
    #[derive(Deserialize)]
    pub struct SignDetailRaw {
        #[serde(rename = "ifPhoto")]
        is_photo_sign: i64,
        #[serde(rename = "ifRefreshEwm")]
        is_refresh_qrcode: i64,
        #[serde(rename = "signCode")]
        sign_code: Option<String>,
    }
    impl From<SignDetailRaw> for SignDetail {
        #[inline]
        fn from(value: SignDetailRaw) -> Self {
            SignDetail::new(
                value.is_photo_sign,
                value.is_refresh_qrcode,
                value.sign_code,
            )
        }
    }
    #[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone, Getset2)]
    #[getset2(get_ref(pub))]
    pub struct CloudItem {
        pub(super) name: String,
        pub(super) object_id: String,
    }
    impl CloudItem {
        #[inline]
        pub fn upload_temporary<TypesProtocol, R>(
            agent: &Agent,
            uid: &str,
            file_name: impl AsRef<std::path::Path>,
            reader: R,
        ) -> Result<Self, AgentError>
        where
            R: std::io::Read,
            TypesProtocol: TypesProtocolTrait,
        {
            let token = TypesProtocol::get_netdisk_token(agent)?;
            let object_id = TypesProtocol::upload_temporary_object(
                agent,
                &token,
                uid,
                file_name.as_ref(),
                reader,
            )?;
            Ok(Self {
                name: file_name.as_ref().to_string_lossy().into_owned(),
                object_id,
            })
        }
    }
    pub struct CloudRoot {
        pub(super) enc: String,
        pub(super) root_id: String,
    }
    impl CloudRoot {
        #[inline]
        pub fn ls<TypesProtocol: TypesProtocolTrait>(
            &self,
            agent: &Agent,
        ) -> Result<
            impl DoubleEndedIterator<Item = CloudItem>
            + std::fmt::Debug
            + Send
            + Sync
            + std::iter::FusedIterator,
            AgentError,
        > {
            TypesProtocol::pan_list(agent, &self.root_id, &self.enc)
        }
    }
}
pub use types_::*;
pub trait TypesProtocolTrait {
    #[inline]
    fn active_list_url() -> &'static str {
        TypesProtocol::ACTIVE_LIST
    }
    #[inline]
    fn get_location_log_url() -> &'static str {
        TypesProtocol::GET_LOCATION_LOG
    }
    #[inline]
    fn sign_detail_url() -> &'static str {
        TypesProtocol::SIGN_DETAIL
    }
    #[inline]
    fn pan_chaoxing_url() -> &'static str {
        TypesProtocol::PAN_CHAOXING
    }
    #[inline]
    fn pan_list_url() -> &'static str {
        TypesProtocol::PAN_LIST
    }
    #[inline]
    fn pan_token_url() -> &'static str {
        TypesProtocol::PAN_TOKEN
    }
    #[inline]
    fn pan_upload_url() -> &'static str {
        TypesProtocol::PAN_UPLOAD
    }
    /// 查询课程活动。
    fn active_list(
        client: &Agent,
        (course_id, class_id): (i64, impl Display),
    ) -> Result<ActivityListRaw, AgentError> {
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();
        let url = Self::active_list_url();
        let url = format!(
            "{url}?fid=0&courseId={course_id}&classId={class_id}&showNotStartedActive=0&_={time}",
        );
        debug!("{url}");
        let r = client.get(&url).call()?;
        let r = {
            #[cfg(debug_assertions)]
            {
                let mut r = r;
                let r = r.body_mut().read_to_string().unwrap();
                match serde_json::from_str(&r) {
                    Ok(r) => r,
                    Err(e) => {
                        log::error!("{course_id}/{class_id}");
                        log::error!("{r}");
                        log::error!("{e:?}");
                        panic!()
                    }
                }
            }
            #[cfg(not(debug_assertions))]
            {
                r.into_body().read_json().log_unwrap()
            }
        };
        Ok(r)
    }

    // 获取位置信息列表
    #[inline]
    fn get_location_log(
        session: &Agent,
        (course_id, class_id): (i64, impl Display),
    ) -> Result<LocationLogData, AgentError> {
        let url = Self::get_location_log_url();
        let r = session
            .get(&format!(
                "{url}?DB_STRATEGY=COURSEID&STRATEGY_PARA=courseId&courseId={course_id}&classId={class_id}",
            ))
            .call()?;
        Ok(r.into_body().read_json().log_unwrap())
    }

    // 签到信息获取
    #[inline]
    fn sign_detail(client: &Agent, active_id: &str) -> Result<SignDetailRaw, AgentError> {
        let url = Self::sign_detail_url();
        let url = format!("{url}?activePrimaryId={active_id}&type=1",);
        debug!("{url}");
        Ok(client
            .get(&url)
            .call()?
            .into_body()
            .read_json()
            .log_unwrap())
    }
    // 超星网盘页
    #[inline]
    fn chaoxing_netdisk_root(client: &Agent) -> Result<CloudRoot, AgentError> {
        let url = Self::pan_chaoxing_url();
        let r = client.get(url).call()?;
        let r_text = r.into_body().read_to_string().log_unwrap();
        let start_of_enc = r_text
            .find("enc =\"")
            .unwrap_or_else(|| panic!("{}:{}:未找到“enc =”，无法寻找云盘照片。", file!(), line!()))
            + 6;
        let end_of_enc = r_text[start_of_enc..r_text.len()]
            .find('"')
            .unwrap_or_else(|| {
                panic!(
                    "{}:{}:未找到“enc =”的结尾段，无法寻找云盘照片。",
                    file!(),
                    line!()
                )
            })
            + start_of_enc;
        let enc = &r_text[start_of_enc..end_of_enc];
        let start_of_root_dir = r_text.find("_rootdir = \"").unwrap_or_else(|| {
            panic!(
                "{}:{}:未找到“_rootdir = ”，无法寻找云盘照片。",
                file!(),
                line!()
            )
        }) + 12;
        let end_of_root_dir = r_text[start_of_root_dir..r_text.len()]
            .find('"')
            .unwrap_or_else(|| {
                panic!(
                    "{}:{}:未找到“_rootdir = ”的结尾段，无法寻找云盘照片。",
                    file!(),
                    line!()
                )
            })
            + start_of_root_dir;
        let root_id = &r_text[start_of_root_dir..end_of_root_dir];
        Ok(CloudRoot {
            enc: enc.to_owned(),
            root_id: root_id.to_owned(),
        })
    }

    // 网盘列表
    fn pan_list(
        client: &Agent,
        parent_id: &str,
        enc: &str,
    ) -> Result<
        impl DoubleEndedIterator<Item = CloudItem> + Debug + Send + Sync + std::iter::FusedIterator,
        AgentError,
    > {
        let url = Self::pan_list_url();
        #[derive(serde::Deserialize, Debug)]
        pub struct CloudItemRaw {
            name: String,
            #[serde(rename = "objectId")]
            object_id: Option<String>,
        }
        #[derive(serde::Deserialize)]
        struct TmpR {
            list: Vec<CloudItemRaw>,
        }
        let r = client
            .post(&format!(
                "{url}?puid=0&shareid=0&parentId={parent_id}&page=1&size=50&enc={enc}"
            ))
            .send_empty()?;
        Ok(r.into_body()
            .read_json::<TmpR>()
            .log_unwrap()
            .list
            .into_iter()
            .filter_map(|CloudItemRaw { name, object_id }| {
                object_id.map(|object_id| CloudItem { name, object_id })
            }))
    }

    // 获取超星云盘的 token
    #[inline]
    fn get_netdisk_token(client: &Agent) -> Result<String, AgentError> {
        #[derive(serde::Deserialize)]
        struct TmpR {
            #[serde(rename = "_token")]
            token: String,
        }
        let url = Self::pan_token_url();
        let r = client.get(url).call()?;
        Ok(r.into_body().read_json::<TmpR>().log_unwrap().token)
    }

    // 网盘上传接口
    fn upload_temporary_object<R: std::io::Read>(
        agent: &Agent,
        token: &str,
        uid: &str,
        file_name: impl AsRef<Path>,
        reader: R,
    ) -> Result<String, AgentError> {
        use crate::multipart::{Field, PreparedFields};
        let url = Self::pan_upload_url();
        let file_as_path: &Path = file_name.as_ref();
        let file_ext = file_as_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        let mime = mime_guess::from_ext(file_ext).first_or_octet_stream();
        #[repr(transparent)]
        struct DisplayOsString(OsString);
        impl Borrow<DisplayOsStr> for DisplayOsString {
            #[inline]
            fn borrow(&self) -> &DisplayOsStr {
                let s: &OsStr = self.0.borrow();
                unsafe { &*(s as *const _ as *const DisplayOsStr) }
            }
        }
        #[repr(transparent)]
        struct DisplayOsStr(OsStr);
        impl ToOwned for DisplayOsStr {
            type Owned = DisplayOsString;

            #[inline]
            fn to_owned(&self) -> Self::Owned {
                DisplayOsString(self.0.to_owned())
            }
        }
        impl<'a> From<&'a DisplayOsStr> for Cow<'a, DisplayOsStr> {
            /// Converts the string reference into a [`Cow::Borrowed`].
            #[inline]
            fn from(s: &'a DisplayOsStr) -> Cow<'a, DisplayOsStr> {
                Cow::Borrowed(s)
            }
        }
        impl Display for DisplayOsStr {
            #[inline]
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                <_ as std::fmt::Display>::fmt(&self.0.display(), f)
            }
        }
        let mut fields = Vec::<Field<DisplayOsStr>>::default();
        Field::add_stream(
            &mut fields,
            "file",
            reader,
            Some(unsafe { &*(file_as_path.as_os_str() as *const _ as *const _) }),
            Some(mime),
        );
        Field::add_text(&mut fields, "puid", uid);
        let mut multipart = PreparedFields::from_fields(&mut fields).unwrap();
        #[derive(serde::Deserialize)]
        struct Tmp {
            #[serde(rename = "objectId")]
            object_id: String,
        }
        let r = agent
            .post(&format!("{url}?_from=mobilelearn&_token={token}"))
            .header(
                "Content-Type",
                &format!("multipart/form-data; boundary={}", multipart.get_boundary()),
            )
            .send(SendBody::from_reader(&mut multipart))?;
        Ok(r.into_body().read_json::<Tmp>().log_unwrap().object_id)
    }
}

pub struct TypesProtocol;

impl TypesProtocol {
    /// 查询活动
    pub const ACTIVE_LIST: &'static str =
        "https://mobilelearn.chaoxing.com/v2/apis/active/student/activelist";
    /// 获取位置信息列表
    pub const GET_LOCATION_LOG: &'static str =
        "https://mobilelearn.chaoxing.com/v2/apis/sign/getLocationLog";
    /// 签到信息获取
    pub const SIGN_DETAIL: &'static str = "https://mobilelearn.chaoxing.com/newsign/signDetail";
    /// 超星网盘页
    pub const PAN_CHAOXING: &'static str = "https://pan-yz.chaoxing.com";
    /// 网盘列表
    pub const PAN_LIST: &'static str = "https://pan-yz.chaoxing.com/opt/listres";
    /// 获取超星云盘的 token
    pub const PAN_TOKEN: &'static str = "https://pan-yz.chaoxing.com/api/token/uservalid";
    /// 网盘上传接口
    pub const PAN_UPLOAD: &'static str = "https://pan-yz.chaoxing.com/upload";
}

impl TypesProtocolTrait for TypesProtocol {}
