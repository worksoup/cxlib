use crate::protocol;
use crate::user::session::Session;
use serde::Deserialize;
use std::fs::File;
use std::path::Path;

// TODO: 删除 unwrap
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Photo {
    object_id: String,
}

impl Photo {
    async fn 通过某种规则从网盘获取object_id(
        session: &Session,
        p: impl Fn(&str) -> bool,
    ) -> Result<Option<String>, ureq::Error> {
        let 响应 = protocol::pan_chaoxing(session).await?;
        let 响应的文本 = 响应.into_string().unwrap();
        let start_of_enc = 响应的文本.find("enc =\"").unwrap() + 6;
        let end_of_enc = 响应的文本[start_of_enc..响应的文本.len()]
            .find('"')
            .unwrap()
            + start_of_enc;
        let enc = &响应的文本[start_of_enc..end_of_enc];
        let start_of_root_dir = 响应的文本.find("_rootdir = \"").unwrap() + 12;
        let end_of_root_dir = 响应的文本[start_of_root_dir..响应的文本.len()]
            .find('"')
            .unwrap()
            + start_of_root_dir;
        let parent_id = &响应的文本[start_of_root_dir..end_of_root_dir];
        let r = protocol::pan_list(session, parent_id, enc).await?;
        #[derive(Deserialize)]
        #[allow(non_snake_case)]
        struct CloudFile {
            name: String,
            objectId: Option<String>,
        }
        #[derive(Deserialize)]
        struct TmpR {
            list: Vec<CloudFile>,
        }
        let r: TmpR = r.into_json()?;
        for item in r.list {
            if p(&item.name) {
                return Ok(item.objectId);
            }
        }
        Ok(None)
    }
    pub fn get_object_id(&self) -> &str {
        &self.object_id
    }
    pub async fn new(session: &Session, file: &File, file_name: &str) -> Self {
        let object_id = session.upload_image(file, file_name).await.unwrap();
        Self { object_id }
    }
    pub async fn 默认(session: &Session) -> Option<Self> {
        Self::从网盘获取(session, |a| a == "1.png" || a == "1.jpg").await
    }
    pub async fn 从网盘获取(session: &Session, p: impl Fn(&str) -> bool) -> Option<Self> {
        let object_id = Self::通过某种规则从网盘获取object_id(session, p)
            .await
            .unwrap();
        if let Some(object_id) = object_id {
            Some(Self { object_id })
        } else {
            None
        }
    }
    pub async fn 上传文件获取(session: &Session, file_path: impl AsRef<Path>) -> Self {
        let f = File::open(&file_path).unwrap();
        let file_name = file_path.as_ref().file_name().unwrap().to_str().unwrap();
        Self::new(session, &f, file_name).await
    }
}
