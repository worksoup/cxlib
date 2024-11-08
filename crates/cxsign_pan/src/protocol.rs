use crate::multipart::{Field, PreparedFields};
use cxsign_protocol::Protocol;
use std::fs::File;
use std::path::Path;
use ureq::{Agent, Response};

// 超星网盘页
pub fn pan_chaoxing(client: &Agent) -> Result<Response, Box<ureq::Error>> {
    Ok(client.get(&Protocol::PanChaoxing.to_string()).call()?)
}

// 网盘列表
pub fn pan_list(client: &Agent, parent_id: &str, enc: &str) -> Result<Response, Box<ureq::Error>> {
    Ok(client
        .post(&format!(
            "{}?puid=0&shareid=0&parentId={parent_id}&page=1&size=50&enc={enc}",
            Protocol::PanList
        ))
        .call()?)
}

// 获取超星云盘的 token
pub fn pan_token(client: &Agent) -> Result<Response, Box<ureq::Error>> {
    Ok(client.get(&Protocol::PanToken.to_string()).call()?)
}

// 网盘上传接口
pub fn pan_upload(
    client: &Agent,
    file: &File,
    uid: &str,
    token: &str,
    file_name: &str,
) -> Result<Response, Box<ureq::Error>> {
    let file_ext: &Path = file_name.as_ref();
    let file_ext = file_ext.extension().and_then(|s| s.to_str()).unwrap_or("");
    let mime = mime_guess::from_ext(file_ext).first_or_octet_stream();
    let mut fields = Vec::<Field>::default();
    Field::add_stream(&mut fields, "file", file, Some(file_name), Some(mime));
    Field::add_text(&mut fields, "puid", uid);
    let multipart = PreparedFields::from_fields(&mut fields).unwrap();
    Ok(client
        .post(&format!(
            "{}?_from=mobilelearn&_token={token}",
            Protocol::PanUpload,
        ))
        .set(
            "Content-Type",
            &format!("multipart/form-data; boundary={}", multipart.get_boundary()),
        )
        .send(multipart)?)
}
