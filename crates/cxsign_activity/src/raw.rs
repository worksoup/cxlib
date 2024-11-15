use cxsign_types::Course;
use cxsign_utils::get_width_str_should_be;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// # RawSign
///
/// 未分类的课程签到。
///
/// 对于该类型的分类、处理等，请参考 `cxsign_default_impl::sign` 中的相关部分。
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct RawSign {
    pub start_time_mills: u64,
    pub active_id: String,
    pub name: String,
    pub course: Course,
    pub other_id: String,
    pub status_code: i32,
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
            cxsign_utils::time_string_from_mills(self.start_time_mills),
            self.course.get_id(),
            self.course.get_name(),
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
            cxsign_utils::time_string_from_mills(self.start_time_mills),
            width = name_width,
        )
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
