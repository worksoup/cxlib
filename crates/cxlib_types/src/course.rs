use crate::protocol;
use cxlib_error::{LoginError, UnwrapOrLogPanic};
use cxlib_user::Session;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::Entry, HashMap},
    fmt::Display,
    ops::Deref,
};
use ureq::serde_json;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Course {
    id: i64,
    class_id: i64,
    teacher: String,
    image_url: String,
    name: String,
}

impl Display for Course {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "班级号：{}, 课程号: {}, 课程名: {}, 任课教师: {}",
            self.class_id, self.id, self.name, self.teacher
        )
    }
}

impl Course {
    pub fn get_courses<'a, Sessions: Iterator<Item = &'a Session>>(
        sessions: Sessions,
    ) -> Result<HashMap<Course, Vec<Session>>, cxlib_error::Error> {
        let mut courses = HashMap::<_, Vec<_>>::new();
        for session in sessions {
            let courses_ = Course::get_session_courses(session).unwrap_or_else(|e| {
                warn!(
                    "未能获取用户[{}]的课程，错误信息：{e}.",
                    session.get_stu_name()
                );
                Default::default()
            });
            for course in courses_ {
                let entry = courses.entry(course);
                match entry {
                    Entry::Occupied(mut entry) => {
                        entry.get_mut().push(session.clone());
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(vec![session.clone()]);
                    }
                }
            }
        }
        Ok(courses)
    }
    pub fn get_session_courses(session: &Session) -> Result<Vec<Course>, cxlib_error::Error> {
        let r = protocol::back_clazz_data(session.deref())?;
        let courses = Course::get_list_from_response(r)?;
        info!("用户[{}]已获取课程列表。", session.get_stu_name());
        Ok(courses)
    }
    fn get_list_from_response(r: ureq::Response) -> Result<Vec<Course>, cxlib_error::Error> {
        let r: GetCoursesR = r.into_json().unwrap_or_log_panic();
        let mut arr = Vec::new();
        if let Some(channel_list) = r.channel_list {
            for c in channel_list {
                if let Some(data) = c.content.course {
                    for course in data.data {
                        if c.id.is_i64() {
                            arr.push(Course::new(
                                course.id,
                                c.id.as_i64().unwrap(),
                                course.teacher.as_str(),
                                course.image_url.unwrap_or("".into()).as_str(),
                                course.name.as_str(),
                            ))
                        }
                    }
                }
            }
            Ok(arr)
        } else {
            Err(LoginError::LoginExpired(
                "`channelList` 字段为空!".to_string(),
            ))?
        }
    }

    pub fn new(id: i64, class_id: i64, teacher: &str, image_url: &str, name: &str) -> Course {
        Course {
            id,
            class_id,
            teacher: teacher.into(),
            image_url: image_url.into(),
            name: name.into(),
        }
    }
    // fn from_raw(raw: &CourseRaw, class_id: i64) -> Course {
    //     Self {
    //         id: raw.id,
    //         class_id,
    //         teacher: raw.teacher.clone(),
    //         image_url: raw.image_url.clone(),
    //         name: raw.name.clone(),
    //     }
    // }
    pub fn get_id(&self) -> i64 {
        self.id
    }
    pub fn get_class_id(&self) -> i64 {
        self.class_id
    }
    pub fn get_teacher(&self) -> &str {
        &self.teacher
    }
    pub fn get_image_url(&self) -> &str {
        &self.image_url
    }
    pub fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct CourseRaw {
    id: i64,
    #[serde(rename = "teacherfactor")]
    teacher: String,
    #[serde(rename = "imageurl")]
    image_url: Option<String>,
    name: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Courses {
    data: Vec<CourseRaw>,
}

#[derive(Deserialize, Serialize, Debug)]
struct CourseContent {
    course: Option<Courses>,
}

#[derive(Deserialize, Serialize, Debug)]
struct ClassRaw {
    #[serde(rename = "key")]
    id: serde_json::Value,
    content: CourseContent,
}

#[derive(Deserialize, Serialize, Debug)]
struct GetCoursesR {
    #[serde(rename = "channelList")]
    channel_list: Option<Vec<ClassRaw>>,
}
