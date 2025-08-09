mod types {
    use cxlib_error::AgentError;
    use getset2::Getset2;
    use ureq::Agent;

    use crate::collect::NetdiskProtocolTrait;

    #[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone, Getset2)]
    #[getset2(get_ref(pub))]
    pub struct CloudItem {
        pub(super) name: String,
        pub(super) object_id: String,
    }
    impl CloudItem {
        #[cfg(feature = "upload-file")]
        #[inline]
        pub fn upload_temporary<NetdiskProtocol, R>(
            agent: &Agent,
            uid: &str,
            file_name: impl AsRef<std::path::Path>,
            reader: R,
        ) -> Result<Self, AgentError>
        where
            R: std::io::Read,
            NetdiskProtocol: NetdiskProtocolTrait,
        {
            let token = NetdiskProtocol::get_netdisk_token(agent)?;
            let object_id = NetdiskProtocol::upload_temporary_object(
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
        pub fn ls<NetdiskProtocol: NetdiskProtocolTrait>(
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
            NetdiskProtocol::pan_list(agent, &self.root_id, &self.enc)
        }
    }
}

use cxlib_error::AgentError;
use cxlib_error_utils::CxlibResultUtils;
pub use types::*;
use ureq::Agent;
pub trait NetdiskProtocolTrait {
    #[inline]
    fn pan_chaoxing_url() -> &'static str {
        NetdiskProtocol::PAN_CHAOXING
    }
    #[inline]
    fn pan_list_url() -> &'static str {
        NetdiskProtocol::PAN_LIST
    }
    #[inline]
    fn pan_token_url() -> &'static str {
        NetdiskProtocol::PAN_TOKEN
    }
    #[inline]
    fn pan_upload_url() -> &'static str {
        NetdiskProtocol::PAN_UPLOAD
    }
    // 超星网盘页
    #[inline]
    fn chaoxing_netdisk_root(agent: &Agent) -> Result<CloudRoot, AgentError> {
        let url = Self::pan_chaoxing_url();
        let r = agent.get(url).call()?;
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
        impl DoubleEndedIterator<Item = CloudItem>
        + std::fmt::Debug
        + Send
        + Sync
        + std::iter::FusedIterator,
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
    #[cfg(feature = "upload-file")]
    fn upload_temporary_object<R: std::io::Read>(
        agent: &Agent,
        token: &str,
        uid: &str,
        file_name: impl AsRef<std::path::Path>,
        reader: R,
    ) -> Result<String, AgentError> {
        use crate::multipart::{Field, PreparedFields};
        use std::{
            borrow::{Borrow, Cow},
            ffi::{OsStr, OsString},
            path::Path,
        };
        use ureq::SendBody;
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
        impl std::fmt::Display for DisplayOsStr {
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
pub struct NetdiskProtocol;
impl NetdiskProtocol {
    /// 超星网盘页
    pub const PAN_CHAOXING: &'static str = "https://pan-yz.chaoxing.com";
    /// 网盘列表
    pub const PAN_LIST: &'static str = "https://pan-yz.chaoxing.com/opt/listres";
    /// 获取超星云盘的 token
    pub const PAN_TOKEN: &'static str = "https://pan-yz.chaoxing.com/api/token/uservalid";
    /// 网盘上传接口
    pub const PAN_UPLOAD: &'static str = "https://pan-yz.chaoxing.com/upload";
}
impl NetdiskProtocolTrait for NetdiskProtocol {}
