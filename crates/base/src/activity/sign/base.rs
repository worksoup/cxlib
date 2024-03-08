use crate::activity::sign::{
    SignState, GestureSign, LocationSign, NormalSign, PhotoSign, QrCodeSign, Sign, SignDetail,
    SignResult, SignTrait, SigncodeSign,
};
use crate::course::Course;
use crate::location::Location;
use crate::photo::Photo;
use crate::protocol;
use crate::user::session::Session;
use crate::utils::get_width_str_should_be;
use serde::Deserialize;
use ureq::Error;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct BaseSign {
    pub start_timestamp: i64,
    pub active_id: String,
    pub name: String,
    pub course: Course,
    pub other_id: String,
    pub status_code: i32,
    pub sign_detail: SignDetail,
}
impl SignTrait for BaseSign {
    fn is_valid(&self) -> bool {
        let time = std::time::SystemTime::from(
            chrono::DateTime::from_timestamp(self.start_timestamp, 0).unwrap(),
        );
        let one_hour = std::time::Duration::from_secs(7200);
        self.status_code == 1
            && std::time::SystemTime::now().duration_since(time).unwrap() < one_hour
    }
    fn get_attend_info(&self, session: &Session) -> Result<SignState, ureq::Error> {
        let r = crate::protocol::get_attend_info(&session, &self.active_id)?;
        #[derive(Deserialize)]
        struct Data原始数据 {
            status: i64,
        }
        #[derive(Deserialize)]
        struct 原始数据 {
            data: Data原始数据,
        }
        let 原始数据 {
            data: Data原始数据 { status },
        } = r.into_json().unwrap();
        Ok(status.into())
    }
    fn pre_sign(&self, session: &Session) -> Result<SignResult, ureq::Error> {
        let active_id = self.active_id.as_str();
        let uid = session.get_uid();
        let response_of_pre_sign =
            protocol::pre_sign(session, self.course.clone(), active_id, uid, false, "", "")?;
        println!("用户[{}]预签到已请求。", session.get_stu_name());
        self.预签到_analysis部分(active_id, session, response_of_pre_sign)
    }
    fn sign(&self, session: &Session) -> Result<SignResult, Error> {
        let r = protocol::general_sign(
            session,
            session.get_uid(),
            session.get_fid(),
            session.get_stu_name(),
            self.active_id.as_str(),
        )?;
        Ok(Self::通过文本判断签到结果(
            &r.into_string().unwrap(),
        ))
    }
}

impl BaseSign {
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

    async fn check_signcode(
        session: &Session,
        active_id: &str,
        signcode: &str,
    ) -> Result<bool, ureq::Error> {
        #[derive(Deserialize)]
        struct CheckR {
            #[allow(unused)]
            result: i64,
        }
        let CheckR { result } = crate::protocol::check_signcode(session, active_id, signcode)
            .await?
            .into_json()
            .unwrap();
        Ok(result == 1)
    }
}

impl BaseSign {
    pub fn to_sign(self) -> Sign {
        match self.other_id.parse::<u8>().unwrap_or_else(|e| {
            eprintln!("{}", self.other_id);
            eprintln!("{}", self.course.get_name());
            panic!("{e}")
        }) {
            0 => {
                if self.is_photo_sign() {
                    Sign::Photo(PhotoSign {
                        base_sign: self,
                        photo: None,
                    })
                } else {
                    Sign::Normal(NormalSign { base_sign: self })
                }
            }
            1 => Sign::Unknown(self),
            2 => Sign::QrCode(QrCodeSign { base_sign: self }),
            3 => Sign::Gesture(GestureSign { base_sign: self }),
            4 => Sign::Location(LocationSign { base_sign: self }),
            5 => Sign::Signcode(SigncodeSign { base_sign: self }),
            _ => Sign::Unknown(self),
        }
    }
    pub fn display(&self, already_course: bool) {
        let name_width = get_width_str_should_be(self.name.as_str(), 12);
        if already_course {
            println!(
                "id: {}, name: {:>width$}, status: {}, time: {}, ok: {}",
                self.active_id,
                self.name,
                self.status_code,
                chrono::DateTime::from_timestamp(self.start_timestamp, 0).unwrap(),
                self.is_valid(),
                width = name_width,
            );
        } else {
            println!(
                "id: {}, name: {:>width$}, status: {}, time: {}, ok: {}, course: {}/{}",
                self.active_id,
                self.name,
                self.status_code,
                chrono::DateTime::from_timestamp(self.start_timestamp, 0).unwrap(),
                self.is_valid(),
                self.course.get_id(),
                self.course.get_name(),
                width = name_width,
            );
        }
    }

    fn is_photo_sign(&self) -> bool {
        self.sign_detail.is_photo
    }

    pub fn is_refesh_qrcode(&self) -> bool {
        self.sign_detail.is_refresh_qrcode
    }

    pub fn get_二维码签到时的c参数(&self) -> &str {
        &self.sign_detail.c
    }

    fn 预签到_analysis部分(
        &self,
        active_id: &str,
        session: &Session,
        response_of_presign: ureq::Response,
    ) -> Result<SignResult, ureq::Error> {
        let response_of_analysis = protocol::analysis(session, active_id)?;
        let data = response_of_analysis.into_string().unwrap();
        let code = {
            let start_of_code = data.find("code='+'").unwrap() + 8;
            let data = &data[start_of_code..data.len()];
            let end_of_code = data.find('\'').unwrap();
            &data[0..end_of_code]
        };
        #[cfg(debug_assertions)]
        println!("code: {code:?}");
        let _response_of_analysis2 = protocol::analysis2(session, code)?;
        #[cfg(debug_assertions)]
        println!(
            "analysis 结果：{}",
            _response_of_analysis2.into_string().unwrap()
        );
        let pre_sign_status = {
            let html = response_of_presign.into_string().unwrap();
            #[cfg(debug_assertions)]
            println!("预签到请求结果：{html}");
            if let Some(start_of_statuscontent_h1) = html.find("id=\"statuscontent\"") {
                let html = &html[start_of_statuscontent_h1 + 19..html.len()];
                let end_of_statuscontent_h1 = html.find('<').unwrap();
                let id为statuscontent的h1的内容 = html[0..end_of_statuscontent_h1].trim();
                if id为statuscontent的h1的内容 == "签到成功" {
                    SignResult::Sussess
                } else {
                    SignResult::Fail {
                        msg: id为statuscontent的h1的内容.into(),
                    }
                }
            } else {
                SignResult::Fail { msg: html.into() }
            }
        };
        std::thread::sleep(std::time::Duration::from_millis(500));
        Ok(pre_sign_status)
    }

    pub fn 预签到_对于有刷新二维码签到(
        &self,
        c: &str,
        enc: &str,
        session: &Session,
    ) -> Result<SignResult, ureq::Error> {
        let active_id = self.active_id.as_str();
        let uid = session.get_uid();
        let response_of_presign =
            protocol::pre_sign(session, self.course.clone(), active_id, uid, true, c, enc)?;
        println!("用户[{}]预签到已请求。", session.get_stu_name());
        self.预签到_analysis部分(active_id, session, response_of_presign)
    }
}

impl BaseSign {
    pub async fn 作为普通签到处理(
        &self,
        session: &Session,
    ) -> Result<SignResult, ureq::Error> {
        let r = protocol::general_sign(
            session,
            session.get_uid(),
            session.get_fid(),
            session.get_stu_name(),
            self.active_id.as_str(),
        )?;
        Ok(Self::通过文本判断签到结果(
            &r.into_string().unwrap(),
        ))
    }
    pub async fn 作为签到码签到处理(
        &self,
        session: &Session,
        signcode: &str,
    ) -> Result<SignResult, ureq::Error> {
        if Self::check_signcode(session, &self.active_id, signcode).await? {
            let r = protocol::signcode_sign(
                session,
                session.get_uid(),
                session.get_fid(),
                session.get_stu_name(),
                self.active_id.as_str(),
                signcode,
            )?;
            Ok(Self::通过文本判断签到结果(
                &r.into_string().unwrap(),
            ))
        } else {
            Ok(SignResult::Fail {
                msg: "签到码或手势不正确".into(),
            })
        }
    }
    pub async fn 作为位置签到处理(
        &self,
        address: &Location,
        session: &Session,
        是否限定位置: bool,
    ) -> Result<SignResult, ureq::Error> {
        let r = protocol::location_sign(
            session,
            session.get_uid(),
            session.get_fid(),
            session.get_stu_name(),
            address,
            self.active_id.as_str(),
            是否限定位置,
        )?;
        Ok(Self::通过文本判断签到结果(
            &r.into_string().unwrap(),
        ))
    }
    pub fn 作为拍照签到处理(
        &self,
        photo: &Photo,
        session: &Session,
    ) -> Result<SignResult, ureq::Error> {
        let r = protocol::photo_sign(
            session,
            session.get_uid(),
            session.get_fid(),
            session.get_stu_name(),
            self.active_id.as_str(),
            photo.get_object_id(),
        )?;
        Ok(Self::通过文本判断签到结果(
            &r.into_string().unwrap(),
        ))
    }
    pub async fn 作为二维码签到处理(
        &self,
        enc: &str,
        address: &Location,
        session: &Session,
    ) -> Result<SignResult, ureq::Error> {
        let r = protocol::qrcode_sign(
            session,
            enc,
            session.get_uid(),
            session.get_fid(),
            session.get_stu_name(),
            self.active_id.as_str(),
            address,
        )?;
        Ok(Self::通过文本判断签到结果(
            &r.into_string().unwrap(),
        ))
    }
}

impl BaseSign {
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
