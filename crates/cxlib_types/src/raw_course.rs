use crate::{ClassInfo, Course};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RawCourse {
    id: i64,
    #[serde(rename = "teacherfactor")]
    teacher: String,
    #[serde(rename = "imageurl")]
    image_url: Option<String>,
    name: String,
}

impl RawCourse {
    // #[inline]
    // pub fn none() -> Self {
    //     Self {
    //         id: -1,
    //         teacher: "".to_string(),
    //         image_url: None,
    //         name: "".to_string(),
    //     }
    // }
    #[inline]
    pub fn into_course(self, info: ClassInfo) -> Course {
        Course::new(self, info)
    }
    #[inline]
    pub fn new(id: i64, teacher: String, image_url: Option<String>, name: String) -> RawCourse {
        RawCourse {
            id,
            teacher,
            image_url,
            name,
        }
    }
    #[inline]
    pub fn id(&self) -> i64 {
        self.id
    }
    #[inline]
    pub fn teacher(&self) -> &str {
        &self.teacher
    }
    #[inline]
    pub fn image_url(&self) -> Option<&str> {
        self.image_url.as_ref().map(AsRef::as_ref)
    }
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }
}
