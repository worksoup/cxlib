use crate::utils::PPTSignHelper;
use cxsign_protocol::ProtocolItem;
use cxsign_types::Location;
use cxsign_user::Session;
use ureq::Response;

// 签到
pub fn general_sign_url(session: &Session, active_id: &str) -> PPTSignHelper {
    let uid = session.get_uid();
    let fid = session.get_fid();
    let stu_name = session.get_stu_name();
    format!("{}?activeId={active_id}&uid={uid}&clientip=&latitude=-1&longitude=-1&appType=15&fid={fid}&name={stu_name}", ProtocolItem::PptSign).into()
}
pub fn photo_sign_url(session: &Session, active_id: &str, object_id: &str) -> PPTSignHelper {
    let uid = session.get_uid();
    let fid = session.get_fid();
    let stu_name = session.get_stu_name();
    // NOTE 存疑。
    format!("{}?activeId={active_id}&uid={uid}&clientip=&useragent=&latitude=-1&longitude=-1&appType=15&fid={fid}&objectId={object_id}&name={}", ProtocolItem::PptSign, percent_encoding::utf8_percent_encode(stu_name, percent_encoding::NON_ALPHANUMERIC)).into()
}

pub fn qrcode_sign_url(
    session: &Session,
    enc: &str,
    active_id: &str,
    location: Option<&Location>,
) -> PPTSignHelper {
    let uid = session.get_uid();
    let fid = session.get_fid();
    let stu_name = session.get_stu_name();
    // TODO: 存疑。
    if let Some(location) = location {
        let (addr, lat, lon, alt) = (
            location.get_addr(),
            location.get_lat(),
            location.get_lon(),
            location.get_alt(),
        );
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
    session: &Session,
    location: &Location,
    active_id: &str,
    is_auto_location: bool,
) -> PPTSignHelper {
    let uid = session.get_uid();
    let fid = session.get_fid();
    let stu_name = session.get_stu_name();
    let address = location.get_addr();
    let lat = location.get_lat();
    let lon = location.get_lon();
    let if_tijiao = if is_auto_location { 1 } else { 0 };
    format!("{}?name={stu_name}&address={address}&activeId={active_id}&uid={uid}&clientip=&latitude={lat}&longitude={lon}&fid={fid}&appType=15&ifTiJiao={if_tijiao}", ProtocolItem::PptSign).into()
}

pub fn signcode_sign_url(session: &Session, active_id: &str, signcode: &str) -> PPTSignHelper {
    let uid = session.get_uid();
    let fid = session.get_fid();
    let stu_name = session.get_stu_name();
    format!("{}?activeId={active_id}&uid={uid}&clientip=&latitude=-1&longitude=-1&appType=15&fid={fid}&name={stu_name}&signCode={signcode}", ProtocolItem::PptSign).into()
}

pub fn general_sign(session: &Session, active_id: &str) -> Result<Response, Box<ureq::Error>> {
    general_sign_url(session, active_id).get(session)
}

pub fn photo_sign(
    session: &Session,
    active_id: &str,
    object_id: &str,
) -> Result<Response, Box<ureq::Error>> {
    photo_sign_url(session, active_id, object_id).get(session)
}
pub fn qrcode_sign(
    session: &Session,
    enc: &str,
    active_id: &str,
    location: Option<&Location>,
) -> Result<Response, Box<ureq::Error>> {
    qrcode_sign_url(session, enc, active_id, location).get(session)
}
pub fn location_sign(
    session: &Session,
    location: &Location,
    active_id: &str,
    is_auto_location: bool,
) -> Result<Response, Box<ureq::Error>> {
    location_sign_url(session, location, active_id, is_auto_location).get(session)
}
pub fn signcode_sign(
    session: &Session,
    active_id: &str,
    signcode: &str,
) -> Result<Response, Box<ureq::Error>> {
    signcode_sign_url(session, active_id, signcode).get(session)
}
