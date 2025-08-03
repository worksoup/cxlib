use crate::{
    AccountTable, BinCode, NormalTableTrait, StoreError, TableDefinitionTrait,
    database_guard::DatabaseGuard,
};
use bincode::{Decode, Encode};
use cxlib_error_utils::{CxlibResultUtils, MaybeFatalError};
use cxlib_internal::{
    protocol::collect::UserProtocolTrait,
    types::{__private::UnhandledGeoaddr, Course, CourseInfo, Session},
};
use log::warn;
use redb::{ReadTransaction, ReadableTable, Table};
use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
};

#[derive(Debug, Clone, Default, Copy)]
pub struct CourseTable;
type CourseStoreData = (CourseInfo, CourseData);
type CourseStoreDataWithSessions<UserProtocol> =
    (CourseInfo, CourseData, Vec<Session<UserProtocol>>);
impl CourseTable {
    /// 从缓存中获取课程与会话。
    #[inline]
    pub fn get_courses_with_sessions<LoginSolver, UserProtocol>(
        r_cxt: &ReadTransaction,
    ) -> Result<HashMap<Course, CourseStoreDataWithSessions<UserProtocol>>, StoreError>
    where
        UserProtocol: UserProtocolTrait + 'static,
    {
        let sessions = AccountTable::load_all_sessions(r_cxt)?;
        let course_table = Self::read(r_cxt)?;
        let r = Self::get_courses_with_current_sessions(&course_table, &sessions);
        drop(course_table);
        Ok(r?.collect())
    }
    #[inline]
    pub fn courses_to_course_sessions_map_with_current_sessions<UserProtocol>(
        courses: impl IntoIterator<Item = (Course, CourseStoreData)>,
        sessions: &HashMap<String, Session<UserProtocol>>,
    ) -> impl Iterator<Item = (Course, CourseStoreDataWithSessions<UserProtocol>)> {
        courses.into_iter().map(move |(course, (info, data))| {
            let sessions = data
                .users
                .iter()
                .filter_map(|uid| sessions.get(uid))
                .cloned()
                .collect::<Vec<_>>();
            (course, (info, data, sessions))
        })
    }
    /// 从缓存中获取课程与会话。
    #[inline]
    pub fn get_courses_with_current_sessions<
        's,
        UserProtocol,
        T: ReadableTable<<Self as TableDefinitionTrait>::Key, <Self as TableDefinitionTrait>::Value>,
    >(
        table: &T,
        sessions: &'s HashMap<String, Session<UserProtocol>>,
    ) -> Result<
        impl Iterator<Item = (Course, CourseStoreDataWithSessions<UserProtocol>)>
        // 个人理解：指定捕获列表，否则将捕获 table 的生命周期参数，导致该不透明类型依赖于该生命周期。
        // 但实际上并不依赖。
        + use<'s, UserProtocol, T>,
        StoreError,
    > {
        Ok(Self::courses_to_course_sessions_map_with_current_sessions(
            Self::get_courses(table)?,
            sessions,
        ))
    }
}
impl CourseTable {
    pub fn insert_course(
        table: &mut redb::Table<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
        course: impl Borrow<Course>,
        course_data: impl Borrow<CourseStoreData>,
    ) -> Result<Option<CourseStoreData>, StoreError> {
        Ok(table.insert(course, course_data)?.map(|v| v.value()))
    }
    pub fn merge_course(
        table: &mut redb::Table<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
        course: impl Borrow<Course>,
        course_data: CourseStoreData,
        update_info: bool,
    ) -> Result<CourseStoreData, StoreError> {
        let course_data =
            if let Some((old_info, old_data)) = Self::get_course(table, course.borrow())? {
                let (info, data) = course_data;
                fn merge<T: std::cmp::Eq + std::hash::Hash>(old: Vec<T>, new: Vec<T>) -> Vec<T> {
                    let mut new = new.into_iter().collect::<HashSet<_>>();
                    new.extend(old);
                    new.into_iter().collect()
                }
                let users = merge(old_data.users, data.users);
                let locations = merge(old_data.locations, data.locations);

                (
                    if update_info { info } else { old_info },
                    CourseData::new(data.recently_used_timestamp_secs, users, locations),
                )
            } else {
                course_data
            };
        Self::insert_course(table, course, course_data.clone())?;
        Ok(course_data)
    }
    pub fn update_users<'s>(
        db: &mut DatabaseGuard,
        course: &Course,
        users: impl IntoIterator<Item = &'s (impl Borrow<str> + 's)>,
    ) -> Result<Option<CourseStoreData>, StoreError> {
        let course_data = db.read_once(|r_cxt| {
            let r = Self::read(r_cxt)?;
            Self::get_course(&r, course)
        })?;
        if let Some(course_data) = course_data {
            let mut course_data = course_data;
            let users = users.into_iter().map(Borrow::borrow).collect();
            course_data.1.set_users(users);
            db.write_once(|w_cxt| {
                let mut w = Self::write(w_cxt).log_unwrap();
                Self::insert_course(&mut w, course, course_data)
            })
        } else {
            Ok(None)
        }
    }
    /// 更新最近活动时间，当入参大于原值才会更新。
    ///
    /// 注意，u64::MAX 被视为初始值。如果原值为初始值，将无条件更新。
    pub fn update_recently_used_time(
        table: &mut Table<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
        course: &Course,
        recently_used_timestamp_secs: u64,
    ) -> Result<bool, StoreError> {
        let (info, mut data) = {
            let course_data_guard = table.get(course)?;
            let Some(course_data_guard) = course_data_guard else {
                return Ok(false);
            };
            let r = course_data_guard.value();
            drop(course_data_guard);
            r
        };
        if u64::MAX == *data.recently_used_timestamp()
            || recently_used_timestamp_secs > *data.recently_used_timestamp()
        {
            data.set_recently_used_timestamp(recently_used_timestamp_secs);
            table.insert(course, (info, data))?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    #[inline]
    pub fn insert_uid(
        db: &mut DatabaseGuard,
        course: &Course,
        user: &str,
    ) -> Result<(), StoreError> {
        let course_data = db.read_once(|r_cxt| {
            let r = Self::read(r_cxt)?;
            Self::get_course(&r, course)
        })?;
        if let Some(course_data) = course_data {
            let mut course_data = course_data;
            course_data.1.get_users_mut().push(String::from(user));
            db.write_once(|w_cxt| {
                let mut w = Self::write(w_cxt).log_unwrap();
                Self::insert_course(&mut w, course, course_data)
            })?;
        }
        Ok(())
    }
    #[inline]
    pub fn insert_course_or<
        O: Fn(
            &mut redb::Table<
                <Self as TableDefinitionTrait>::Key,
                <Self as TableDefinitionTrait>::Value,
            >,
            &Course,
            &CourseStoreData,
        ) -> Result<Option<CourseStoreData>, StoreError>,
    >(
        table: &mut redb::Table<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
        course: &Course,
        course_data: impl Borrow<CourseStoreData>,
        or: O,
    ) -> Result<Option<CourseStoreData>, StoreError> {
        match Self::insert_course(table, course, course_data.borrow()) {
            Ok(r) => Ok(r),
            Err(e) => {
                warn!("插入数据失败：{e}");
                or(table, course, course_data.borrow())
            }
        }
    }
    pub fn get_course(
        table: &impl ReadableTable<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
        course: impl Borrow<Course>,
    ) -> Result<Option<CourseStoreData>, StoreError> {
        Ok(table.get(course)?.map(|c| c.value()))
    }

    pub fn get_courses(
        table: &impl ReadableTable<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
    ) -> Result<HashMap<Course, CourseStoreData>, StoreError> {
        let mut hash_map = HashMap::new();
        for r in table.iter()? {
            match r {
                Ok(r) => {
                    hash_map.insert(r.0.value(), r.1.value());
                }
                Err(e) => {
                    let e = StoreError::from(e);
                    if e.is_fatal() {
                        return Err(e);
                    }
                    continue;
                }
            }
        }
        Ok(hash_map)
    }
    #[inline]
    pub fn update_course(
        table: &mut redb::Table<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
        course: &Course,
        course_data: impl Borrow<CourseStoreData>,
    ) -> Result<Option<CourseStoreData>, StoreError> {
        Self::insert_course(table, course, course_data)
    }
    #[inline]
    pub fn update_course_or(
        table: &mut redb::Table<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
        course: &Course,
        course_data: CourseStoreData,
        or: impl FnOnce(
            &mut redb::Table<
                <Self as TableDefinitionTrait>::Key,
                <Self as TableDefinitionTrait>::Value,
            >,
            &Course,
            &CourseStoreData,
        ) -> Result<Option<CourseStoreData>, StoreError>,
    ) -> Result<Option<CourseStoreData>, StoreError> {
        match Self::insert_course(table, course, course_data.borrow()) {
            ok @ Ok(_) => ok,
            Err(e) => {
                warn!("插入数据失败：{e}");
                or(table, course, course_data.borrow())
            }
        }
    }
    // pub fn refresh_courses(db: &Database, session: &Session) -> Result<(), StoreError> {
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
impl NormalTableTrait for CourseTable {}
impl TableDefinitionTrait for CourseTable {
    type Key = BinCode<Course>;
    type Value = BinCode<CourseStoreData>;
    type Context<'cxt> = ();
    const NAME: &'static str = "course";
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Decode, Encode, Clone)]
pub struct CourseData {
    recently_used_timestamp_secs: u64,
    users: Vec<String>,
    locations: Vec<UnhandledGeoaddr>,
}
impl CourseData {
    pub fn new(
        recently_used_timestamp_secs: u64,
        users: Vec<String>,
        locations: Vec<UnhandledGeoaddr>,
    ) -> Self {
        Self {
            recently_used_timestamp_secs,
            users,
            locations,
        }
    }
    pub fn recently_used_timestamp(&self) -> &u64 {
        &self.recently_used_timestamp_secs
    }
    pub fn users(&self) -> impl Iterator<Item = &str> {
        self.users.iter().map(String::as_str)
    }
    pub fn locations(&self) -> impl Iterator<Item = &UnhandledGeoaddr> {
        self.locations.iter()
    }
    pub fn decompose(self) -> (u64, Vec<String>, Vec<UnhandledGeoaddr>) {
        let Self {
            recently_used_timestamp_secs,
            users,
            locations,
        } = self;
        (recently_used_timestamp_secs, users, locations)
    }
    pub fn set_recently_used_timestamp(&mut self, recently_used_timestamp_secs: u64) {
        self.recently_used_timestamp_secs = recently_used_timestamp_secs;
    }
    pub fn set_users(&mut self, users: HashSet<&str>) {
        self.users = users.into_iter().map(ToOwned::to_owned).collect();
    }
    pub fn set_locations(&mut self, locations: HashSet<&UnhandledGeoaddr>) {
        self.locations = locations.into_iter().cloned().collect();
    }
    pub fn get_users_mut(&mut self) -> &mut Vec<String> {
        &mut self.users
    }
    pub fn get_locations_mut(&mut self) -> &mut Vec<UnhandledGeoaddr> {
        &mut self.locations
    }
}
impl CourseData {}
// impl CourseData {
//     pub fn get_sessions(&self, db: &Database) -> HashMap<String, Session> {
//         AccountTable::get_sessions_by_uid_list_str(db, &self.uid_list)
//     }
// }
