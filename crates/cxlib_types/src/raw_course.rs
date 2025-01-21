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
    pub fn into_course(self, info: ClassInfo) -> Course {
        Course::new(self, info)
    }
    pub fn new(id: i64, teacher: String, image_url: Option<String>, name: String) -> RawCourse {
        RawCourse {
            id,
            teacher,
            image_url,
            name,
        }
    }
    pub fn id(&self) -> i64 {
        self.id
    }
    pub fn teacher(&self) -> &str {
        &self.teacher
    }
    pub fn image_url(&self) -> Option<&str> {
        self.image_url.as_ref().map(AsRef::as_ref)
    }
    pub fn name(&self) -> &str {
        &self.name
    }
}
