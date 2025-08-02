use crate::session::Session;
use cxlib_error::AgentError;
use cxlib_error_utils::CxlibResultUtils;
use cxlib_protocol::collect::TypesProtocolTrait;
use derive_where::derive_where;
use serde::{Deserialize, Serialize};
use std::{fs::File, marker::PhantomData, path::Path};

// TODO: 删除 unwrap
#[derive_where(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
#[derive(Serialize)]
pub struct Photo<TypesProtocol> {
    object_id: String,
    #[serde(skip)]
    _p: PhantomData<TypesProtocol>,
}
impl<T> Photo<T> {
    pub fn get_object_id(&self) -> &str {
        &self.object_id
    }
}

impl<TypesProtocol: TypesProtocolTrait> Photo<TypesProtocol> {
    #[inline]
    pub fn get_pan_token<U>(session: &Session<U>) -> Result<String, AgentError> {
        let r = TypesProtocol::pan_token(session)?;
        #[derive(Deserialize)]
        struct Tmp {
            #[serde(rename = "_token")]
            token: String,
        }
        let r: Tmp = r.into_body().read_json().log_unwrap();
        Ok(r.token)
    }

    pub fn new<U>(
        session: &Session<U>,
        file: &File,
        file_name: impl AsRef<Path>,
    ) -> Result<Self, AgentError> {
        let token = Self::get_pan_token(session)?;
        let r = TypesProtocol::pan_upload(session, file, session.uid(), &token, file_name)?;
        #[derive(Deserialize)]
        struct Tmp {
            #[serde(rename = "objectId")]
            object_id: String,
        }
        let tmp: Tmp = r.into_body().read_json().log_unwrap();
        Ok(Self {
            object_id: tmp.object_id,
            _p: Default::default(),
        })
    }
    #[inline]
    pub fn default<U>(session: &Session<U>) -> Option<Self> {
        Self::find_in_cxpan(session, |a| a == "1.png" || a == "1.jpg")
            .log_ok()
            .flatten()
    }
    pub fn find_in_cxpan<U>(
        session: &Session<U>,
        p: impl Fn(&str) -> bool,
    ) -> Result<Option<Self>, AgentError> {
        let r = TypesProtocol::pan_chaoxing(session)?;
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
        let parent_id = &r_text[start_of_root_dir..end_of_root_dir];
        let r = TypesProtocol::pan_list(session, parent_id, enc)?;
        #[derive(Deserialize)]
        struct CloudFile {
            name: String,
            #[serde(rename = "objectId")]
            object_id: Option<String>,
        }
        #[derive(Deserialize)]
        struct TmpR {
            list: Vec<CloudFile>,
        }
        let r: TmpR = r.into_body().read_json()?;
        for item in r.list {
            if p(&item.name) {
                return Ok(item.object_id.map(|object_id| Self {
                    object_id,
                    _p: Default::default(),
                }));
            }
        }
        Ok(None)
    }
    #[inline]
    pub fn get_from_file<U>(
        session: &Session<U>,
        file_path: impl AsRef<Path>,
    ) -> Result<Self, AgentError> {
        let f = File::open(&file_path).log_unwrap();
        let file_name = file_path
            .as_ref()
            .file_name()
            .expect("路径中不包含图片文件的名称");
        Self::new(session, &f, file_name)
    }
}
