use crate::store::{AccountTable, DataBase, DataBaseTableTrait};
use cxlib_error::StoreError;
use cxlib_store::StorageTableCommandTrait;
use cxlib_types::{ClassId, ClassInfo, Course, RawCourse, Session};
use log::warn;
use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};

pub struct CourseTable;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CourseData {
    inner: Course,
    recently_used_timestamp_secs: u64,
    uid_list: String,
}
impl CourseData {
    pub fn new(course: Course, uid_list: String, recently_used_timestamp_secs: u64) -> Self {
        Self {
            inner: course,
            recently_used_timestamp_secs,
            uid_list,
        }
    }
    pub fn get_uid_list(&self) -> HashSet<&str> {
        self.uid_list
            .split(',')
            .map(|s| s.trim())
            .collect::<HashSet<&str>>()
    }
}
impl CourseData {
    pub fn as_inner(&self) -> &Course {
        &self.inner
    }
}
impl Deref for CourseData {
    type Target = Course;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
// impl CourseData {
//     pub fn get_sessions(&self, db: &DataBase) -> HashMap<String, Session> {
//         AccountTable::get_sessions_by_uid_list_str(db, &self.uid_list)
//     }
// }
impl CourseTable {
    /// 从缓存中获取课程与会话。
    #[inline]
    pub fn get_courses_with_sessions(db: &DataBase) -> HashMap<CourseData, Vec<Session>> {
        let sessions = AccountTable::get_sessions(db);
        Self::get_courses_with_current_sessions(db, sessions)
    }
    /// 从缓存中获取课程与会话。
    #[inline]
    pub fn get_courses_with_current_sessions(
        db: &DataBase,
        sessions: HashMap<String, Session>,
    ) -> HashMap<CourseData, Vec<Session>> {
        Self::get_courses(db)
            .into_values()
            .map(|course| {
                let sessions = course
                    .uid_list
                    .split(",")
                    .filter_map(|uid| sessions.get(uid))
                    .cloned()
                    .collect::<Vec<_>>();
                (course, sessions)
            })
            .collect()
    }
    pub fn insert_course(db: &DataBase, course: &CourseData) -> Result<(), StoreError> {
        let id: i64 = course.id();
        let clazzid: i64 = course.class_id().into();
        let name: &str = course.name();
        let teacher: &str = course.teacher();
        let image: &str = course.image_url().unwrap_or("");
        let ended = if course.class_ended() { 1 } else { 0 };
        let mut query = db.prepare(format!("INSERT INTO {}(id,clazzid,name,teacher,image,ended,RU,users) values(:id,:clazzid,:name,:teacher,:image,:ended,:RU,:users);", Self::TABLE_NAME)).unwrap();
        query
            .bind::<&[(_, sqlite::Value)]>(
                &[
                    (":id", id.into()),
                    (":clazzid", clazzid.into()),
                    (":name", name.into()),
                    (":teacher", teacher.into()),
                    (":image", image.into()),
                    (":ended", ended.into()),
                    (
                        ":RU",
                        course.recently_used_timestamp_secs.to_string().into(),
                    ),
                    (":FU", course.uid_list.as_str().into()),
                ][..],
            )
            .unwrap();
        query
            .next()
            .map_err(|e| StoreError::ParseError(e.to_string()))
            .map(|_| ())
    }
    pub fn update_users(db: &DataBase, course_id: i64, users: &str) -> Result<(), StoreError> {
        let mut query = db
            .prepare(format!(
                "UPDATE {} SET users=:users WHERE id=:id;",
                Self::TABLE_NAME
            ))
            .unwrap();
        query
            .bind::<&[(_, sqlite::Value)]>(
                &[(":id", course_id.into()), (":users", users.into())][..],
            )
            .unwrap();
        query
            .next()
            .map_err(|e| StoreError::ParseError(e.to_string()))
            .map(|_| ())
    }
    #[inline]
    pub fn insert_uid(db: &DataBase, course_id: i64, uid: &str) -> Result<(), StoreError> {
        let course = Self::get_course(db, course_id)?;
        if let Some(course) = course {
            let mut set = course.uid_list.split(",").collect::<HashSet<&str>>();
            set.insert(uid);
            let uid_list = set.into_iter().collect::<Vec<&str>>().join(",");
            Self::update_users(db, course_id, uid_list.as_str())?;
        }
        Ok(())
    }
    #[inline]
    pub fn insert_course_or<O: Fn(&DataBase, &CourseData) -> Result<(), StoreError>>(
        db: &DataBase,
        course: &CourseData,
        or: O,
    ) -> Result<(), StoreError> {
        match Self::insert_course(db, course) {
            Ok(_) => Ok(()),
            Err(_) => or(db, course),
        }
    }
    pub fn get_course(db: &DataBase, course_id: i64) -> Result<Option<CourseData>, StoreError> {
        let mut query = db
            .prepare(format!(
                "SELECT * FROM {} WHERE id={};",
                Self::TABLE_NAME,
                course_id
            ))
            .unwrap();
        if let Some(row) = query.iter().next() {
            if let Ok(row) = row {
                let id = row.read("id");
                let clazzid = row.read("clazzid");
                let teacher = row.read::<&str, _>("teacher").to_owned();
                let image = row.read::<&str, _>("image").to_owned();
                let name = row.read::<&str, _>("name").to_owned();
                let ended = row.read::<i64, _>("ended");
                let recently_used_timestamp_secs = row.read::<&str, _>("RU").parse().unwrap();
                let uid_list = row.read::<&str, _>("users").to_owned();
                let inner = Course::new(
                    RawCourse::new(id, teacher, Some(image), name),
                    ClassInfo::new(ClassId::Id(clazzid), ended != 0),
                );

                Ok(Some(CourseData {
                    inner,
                    recently_used_timestamp_secs,
                    uid_list,
                }))
            } else {
                Err(StoreError::ParseError(
                    "课程获取失败，没有该课程号的课程。".to_owned(),
                ))
            }
        } else {
            Ok(None)
        }
    }

    pub fn get_courses(db: &DataBase) -> HashMap<i64, CourseData> {
        let mut query = db
            .prepare(format!("SELECT * FROM {};", Self::TABLE_NAME))
            .unwrap();
        let mut courses = HashMap::new();
        for c in query.iter() {
            if let Ok(row) = c {
                let id = row.read("id");
                let clazzid = row.read("clazzid");
                let teacher = row.read::<&str, _>("teacher").to_owned();
                let image = row.read::<&str, _>("image").to_owned();
                let name = row.read::<&str, _>("name").to_owned();
                let ended = row.read::<i64, _>("ended");
                let recently_used_timestamp_secs = row.read::<&str, _>("RU").parse().unwrap();
                let uid_list = row.read::<&str, _>("users").to_owned();
                let inner = Course::new(
                    RawCourse::new(id, teacher, Some(image), name),
                    ClassInfo::new(ClassId::Id(clazzid), ended != 0),
                );
                courses.insert(
                    id,
                    CourseData {
                        inner,
                        recently_used_timestamp_secs,
                        uid_list,
                    },
                );
            } else {
                warn!("课程解析行出错：{c:?}.");
            }
        }
        courses
    }
    pub fn update_course(db: &DataBase, course: &CourseData) -> Result<(), StoreError> {
        let mut query = db.prepare(format!("UPDATE {} SET name=:name,teacher=:teacher,image=:image,ended=:ended,RU=:RU,users=:users WHERE id=:id;", Self::TABLE_NAME)).unwrap();
        query
            .bind::<&[(_, sqlite::Value)]>(
                &[
                    (":id", course.id().into()),
                    (":clazzid", sqlite::Value::Integer(course.class_id().into())),
                    (":name", course.name().into()),
                    (":teacher", course.teacher().into()),
                    (":image", course.image_url().into()),
                    (":ended", if course.class_ended() { 1 } else { 0 }.into()),
                    (
                        ":RU",
                        course.recently_used_timestamp_secs.to_string().into(),
                    ),
                    (":users", course.uid_list.as_str().into()),
                ][..],
            )
            .unwrap();
        query
            .next()
            .map_err(|e| StoreError::ParseError(e.to_string()))
            .map(|_| ())
    }
    #[inline]
    pub fn update_course_or(
        db: &DataBase,
        course: &CourseData,
        or: impl FnOnce(&DataBase, &CourseData) -> Result<(), StoreError>,
    ) -> Result<(), StoreError> {
        match Self::update_course(db, course) {
            Ok(_) => Ok(()),
            Err(e) => {
                warn!("更新课程数据失败：`{e}`.");
                or(db, course)
            }
        }
    }
    // pub fn refresh_courses(db: &DataBase, session: &Session) -> Result<(), StoreError> {
    //     let courses = session.get_courses().map_err(|e| match e {
    //         CourseError::AgentError(e) => StoreError::LoginError(LoginError::AgentError(e)),
    //         CourseError::LoginError(e) => StoreError::LoginError(e),
    //     })?;
    //     for c in courses {
    //         Self::add_course_or(db, &c, |_, _, _, _| {});
    //     }
    //     Ok(())
    // }
}

impl StorageTableCommandTrait<DataBase> for CourseTable {
    #[inline]
    fn init(storage: &DataBase) {
        <Self as DataBaseTableTrait>::init(storage);
    }
    #[inline]
    fn uninit(storage: &DataBase) -> bool {
        !Self::is_existed(storage)
    }
    #[inline]
    fn clear(storage: &DataBase) {
        Self::delete(storage);
    }
    #[inline]
    fn import(storage: &DataBase, content: &str) {
        <Self as DataBaseTableTrait>::import(storage, content);
    }
    #[inline]
    fn export(storage: &DataBase) -> String {
        <Self as DataBaseTableTrait>::export(storage)
    }
}

impl DataBaseTableTrait for CourseTable {
    const TABLE_ARGS: &'static str = "id INTEGER UNIQUE NOT NULL,clazzid INTEGER NOT NULL,name TEXT NOT NULL,teacher TEXT NOT NULL,image TEXT NOT NULL,ended INTEGER NOT NULL,RU TEXT NOT NULL,users TEXT NOT NULL";
    const TABLE_NAME: &'static str = "course";
}
