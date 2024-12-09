use crate::utils::PPTSignHelper;
use crate::ProtocolItem;
use cxlib_error::AgentError;
use ureq::{Agent, Response};

// 签到
pub fn general_sign_url(
    (uid, fid, stu_name): (&str, &str, &str),
    active_id: &str,
) -> PPTSignHelper {
    format!("{}?activeId={active_id}&uid={uid}&clientip=&latitude=-1&longitude=-1&appType=15&fid={fid}&name={stu_name}", ProtocolItem::PptSign).into()
}
pub fn photo_sign_url(
    (uid, fid, stu_name): (&str, &str, &str),
    active_id: &str,
    object_id: &str,
) -> PPTSignHelper {
    // NOTE 存疑。
    format!("{}?activeId={active_id}&uid={uid}&clientip=&useragent=&latitude=-1&longitude=-1&appType=15&fid={fid}&objectId={object_id}&name={}", ProtocolItem::PptSign, percent_encoding::utf8_percent_encode(stu_name, percent_encoding::NON_ALPHANUMERIC)).into()
}

pub fn qrcode_sign_url(
    (uid, fid, stu_name): (&str, &str, &str),
    enc: &str,
    active_id: &str,
    location: Option<(&str, &str, &str, &str)>,
) -> PPTSignHelper {
    // TODO: 存疑。
    if let Some((addr, lat, lon, alt)) = location {
        let location_str = format!(
            r#"{{"result":"1","address":"{addr}","latitude":{lat},"longitude":{lon},"altitude":{alt}}}"#
        );
        let location_str = percent_encoding::utf8_percent_encode(
            &location_str,
            percent_encoding::NON_ALPHANUMERIC,
        )
        .to_string();
        format!(
            r#"{}?enc={enc}&name={stu_name}&activeId={active_id}&uid={uid}&clientip=&location={location_str}&latitude=-1&longitude=-1&fid={fid}&appType=15"#,
            ProtocolItem::PptSign
        )
    } else {
        format!(
            r#"{}?enc={enc}&name={stu_name}&activeId={active_id}&uid={uid}&clientip=&location=&latitude=-1&longitude=-1&fid={fid}&appType=15"#,
            ProtocolItem::PptSign
        )
    }.into()
}
pub fn location_sign_url(
    (uid, fid, stu_name): (&str, &str, &str),
    (addr, lat, lon): (&str, &str, &str),
    active_id: &str,
    is_auto_location: bool,
) -> PPTSignHelper {
    let if_tijiao = if is_auto_location { 1 } else { 0 };
    format!("{}?name={stu_name}&address={addr}&activeId={active_id}&uid={uid}&clientip=&latitude={lat}&longitude={lon}&fid={fid}&appType=15&ifTiJiao={if_tijiao}", ProtocolItem::PptSign).into()
}

pub fn signcode_sign_url(
    (uid, fid, stu_name): (&str, &str, &str),
    active_id: &str,
    signcode: &str,
) -> PPTSignHelper {
    format!("{}?activeId={active_id}&uid={uid}&clientip=&latitude=-1&longitude=-1&appType=15&fid={fid}&name={stu_name}&signCode={signcode}", ProtocolItem::PptSign).into()
}

pub fn general_sign(
    agent: &Agent,
    session: (&str, &str, &str),
    active_id: &str,
) -> Result<Response, AgentError> {
    general_sign_url(session, active_id).get(agent)
}

pub fn photo_sign(
    agent: &Agent,
    session: (&str, &str, &str),
    active_id: &str,
    object_id: &str,
) -> Result<Response, AgentError> {
    photo_sign_url(session, active_id, object_id).get(agent)
}
pub fn qrcode_sign(
    agent: &Agent,
    session: (&str, &str, &str),
    enc: &str,
    active_id: &str,
    location: Option<(&str, &str, &str, &str)>,
) -> Result<Response, AgentError> {
    qrcode_sign_url(session, enc, active_id, location).get(agent)
}
pub fn location_sign(
    agent: &Agent,
    session: (&str, &str, &str),
    location: (&str, &str, &str),
    active_id: &str,
    is_auto_location: bool,
) -> Result<Response, AgentError> {
    location_sign_url(session, location, active_id, is_auto_location).get(agent)
}
pub fn signcode_sign(
    agent: &Agent,
    session: (&str, &str, &str),
    active_id: &str,
    signcode: &str,
) -> Result<Response, AgentError> {
    signcode_sign_url(session, active_id, signcode).get(agent)
}
