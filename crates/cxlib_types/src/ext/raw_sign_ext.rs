use crate::{Session, SignDetail};
use cxlib_base_types::RawSign;
use cxlib_error::AgentError;
use cxlib_protocol::collect::{TypesProtocolTrait, UserProtocolTrait};
use std::{
    fmt::{Display, Formatter},
    time::{Duration, SystemTime},
};

#[inline]
fn get_width_str_should_be(s: &str, width: usize) -> usize {
    use unicode_width::UnicodeWidthStr;
    if UnicodeWidthStr::width(s) > width {
        width
    } else {
        UnicodeWidthStr::width(s) + 12 - s.len()
    }
}

#[inline]
fn time_string_from_mills(mills: u64) -> String {
    #[inline]
    pub fn time_string(t: SystemTime) -> String {
        chrono::DateTime::<chrono::Local>::from(t)
            .format("%+")
            .to_string()
    }
    time_string(std::time::UNIX_EPOCH + Duration::from_millis(mills))
}
pub struct RawSignDisplay<'a>(&'a RawSign);
impl Display for RawSignDisplay<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name_width = get_width_str_should_be(self.0.name(), 12);
        if let Some(mills) = self.0.start_time_mills() {
            write!(
                f,
                "id: {}, name: {:>width$}, status: {}, time: {}, course: {}/{}",
                self.0.active_id(),
                self.0.name(),
                self.0.status_code(),
                time_string_from_mills(*mills),
                self.0.course().id(),
                self.0.course().name(),
                width = name_width,
            )
        } else {
            write!(
                f,
                "id: {}, name: {:>width$}, status: {}, time: no time, course: {}/{}",
                self.0.active_id(),
                self.0.name(),
                self.0.status_code(),
                self.0.course().id(),
                self.0.course().name(),
                width = name_width,
            )
        }
    }
}
pub struct RawSignDisplayWithoutCourse<'a>(&'a RawSign);
impl Display for RawSignDisplayWithoutCourse<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name_width = get_width_str_should_be(self.0.name(), 12);
        if let Some(mills) = self.0.start_time_mills() {
            write!(
                f,
                "id: {}, name: {:>width$}, status: {}, time: {}",
                self.0.active_id(),
                self.0.name(),
                self.0.status_code(),
                time_string_from_mills(*mills),
                width = name_width,
            )
        } else {
            write!(
                f,
                "id: {}, name: {:>width$}, status: {}, time: no time",
                self.0.active_id(),
                self.0.name(),
                self.0.status_code(),
                width = name_width,
            )
        }
    }
}
pub trait RawSignExt {
    fn display<'a>(&'a self) -> RawSignDisplay<'a>;
    fn display_without_course<'a>(&'a self) -> RawSignDisplayWithoutCourse<'a>;
    fn get_sign_detail<TypesProtocol, UserProtocol>(
        &self,
        session: &Session<UserProtocol>,
    ) -> Result<SignDetail, AgentError>
    where
        TypesProtocol: TypesProtocolTrait,
        UserProtocol: UserProtocolTrait;
}
impl RawSignExt for RawSign {
    #[inline]
    fn display<'a>(&'a self) -> RawSignDisplay<'a> {
        RawSignDisplay(self)
    }
    #[inline]
    fn display_without_course<'a>(&'a self) -> RawSignDisplayWithoutCourse<'a> {
        RawSignDisplayWithoutCourse(self)
    }
    #[inline]
    fn get_sign_detail<TypesProtocol, UserProtocol>(
        &self,
        session: &Session<UserProtocol>,
    ) -> Result<SignDetail, AgentError>
    where
        TypesProtocol: TypesProtocolTrait,
        UserProtocol: UserProtocolTrait,
    {
        Ok(TypesProtocol::sign_detail(session, self.active_id())?.into())
    }
}
pub trait RawSignUnusedExt {
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
impl RawSignUnusedExt for RawSign {}
