use crate::{Course, Session, SignDetail};
use cxlib_error::{CxlibResultUtils, SignError};
use cxlib_protocol::collect::types as protocol;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter},
    time::{Duration, SystemTime},
};

pub fn get_width_str_should_be(s: &str, width: usize) -> usize {
    use unicode_width::UnicodeWidthStr;
    if UnicodeWidthStr::width(s) > width {
        width
    } else {
        UnicodeWidthStr::width(s) + 12 - s.len()
    }
}

/// # RawSign
///
/// 未分类的课程签到。
///
/// 对于该类型的分类、处理等，请参考 `cxlib_default_impl::sign` 中的相关部分。
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct RawSign {
    pub start_time_mills: u64,
    pub active_id: String,
    pub name: String,
    pub course: Course,
    pub other_id: String,
    pub status_code: i32,
}
fn time_string_from_mills(mills: u64) -> String {
    pub fn time_string(t: SystemTime) -> String {
        chrono::DateTime::<chrono::Local>::from(t)
            .format("%+")
            .to_string()
    }
    time_string(std::time::UNIX_EPOCH + Duration::from_millis(mills))
}

impl Display for RawSign {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name_width = get_width_str_should_be(self.name.as_str(), 12);
        write!(
            f,
            "id: {}, name: {:>width$}, status: {}, time: {}, course: {}/{}",
            self.active_id,
            self.name,
            self.status_code,
            time_string_from_mills(self.start_time_mills),
            self.course.id(),
            self.course.name(),
            width = name_width,
        )
    }
}

impl RawSign {
    pub fn fmt_without_course_info(&self) -> String {
        let name_width = get_width_str_should_be(self.name.as_str(), 12);
        format!(
            "id: {}, name: {:>width$}, status: {}, time: {}",
            self.active_id,
            self.name,
            self.status_code,
            time_string_from_mills(self.start_time_mills),
            width = name_width,
        )
    }
    pub fn get_sign_detail(active_id: &str, session: &Session) -> Result<SignDetail, SignError> {
        #[derive(Deserialize)]
        struct GetSignDetailR {
            #[serde(rename = "ifPhoto")]
            is_photo_sign: i64,
            #[serde(rename = "ifRefreshEwm")]
            is_refresh_qrcode: i64,
            #[serde(rename = "signCode")]
            sign_code: Option<String>,
        }
        let r = protocol::sign_detail(session, active_id)?;
        let GetSignDetailR {
            is_photo_sign,
            is_refresh_qrcode,
            sign_code,
        } = r.into_json().log_unwrap();
        Ok(SignDetail::new(is_photo_sign, is_refresh_qrcode, sign_code))
    }
    pub fn detail(&self, session: &Session) -> Result<SignDetail, SignError> {
        Self::get_sign_detail(&self.active_id, session)
    }
}
impl RawSign {
    // pub fn speculate_type_by_text(text: &str) -> Sign {
    //     if text.contains("拍照") {
    //         Sign::Photo
    //     } else if text.contains("位置") {
    //         Sign::Location
    //     } else if text.contains("二维码") {
    //         Sign::QrCode
    //     } else if text.contains("手势") {
    //         // ?
    //         Sign::Gesture
    //     } else if text.contains("签到码") {
    //         // ?
    //         Sign::SignCode
    //     } else {
    //         Sign::Normal
    //     }
    // }

    // pub async fn chat_group_pre_sign(
    //     &self,
    //     chat_id: &str,
    //     tuid: &str,
    //     session: &Struct签到会话,
    // ) -> Result<(), ureq::Error> {
    //     let id = self.活动id.as_str();
    //     let uid = session.get_uid();
    //     let _r = protocol::chat_group_pre_sign(session, id, uid, chat_id, tuid).await?;
    //     Ok(())
    // }

    // pub async fn chat_group_general_sign(
    //     &self,
    //     session: &Struct签到会话,
    // ) -> Result<(), ureq::Error> {
    //     let r =
    //         protocol::chat_group_general_sign(session, self.活动id.as_str(), session.get_uid())
    //             .await?;
    //     println!("{:?}", r.text().await.unwrap());
    //     Ok(())
    // }

    // pub async fn chat_group_signcode_sign(
    //     &self,
    //     session: &Struct签到会话,
    //     signcode: &str,
    // ) -> Result<(), ureq::Error> {
    //     let r = protocol::chat_group_signcode_sign(
    //         session,
    //         self.活动id.as_str(),
    //         session.get_uid(),
    //         signcode,
    //     )
    //     .await?;
    //     println!("{:?}", r.text().await.unwrap());
    //     Ok(())
    // }

    // pub async fn chat_group_location_sign(
    //     &self,
    //     address: &Struct位置,
    //     session: &Struct签到会话,
    // ) -> Result<(), ureq::Error> {
    //     let r = protocol::chat_group_location_sign(
    //         session,
    //         address.get_地址(),
    //         self.活动id.as_str(),
    //         session.get_uid(),
    //         address.get_纬度(),
    //         address.get_经度(),
    //     )
    //     .await?;
    //     println!("{:?}", r.text().await.unwrap());
    //     Ok(())
    // }

    // pub async fn chat_group_photo_sign(
    //     &self,
    //     photo: &Struct在线图片,
    //     session: &Struct签到会话,
    // ) -> Result<(), ureq::Error> {
    //     let r = protocol::chat_group_photo_sign(
    //         session,
    //         self.活动id.as_str(),
    //         session.get_uid(),
    //         photo.get_object_id(),
    //     )
    //     .await?;
    //     println!("{:?}", r.text().await.unwrap());
    //     Ok(())
    // }
}
