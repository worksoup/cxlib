use crate::StoreError;
use cxlib_internal::types::{__private::UnhandledGeoaddr, Course};
use log::{debug, warn};
use std::{borrow::Borrow, fmt::Display};
use try_from_with_context::TryFromWithContext;

#[derive(Debug, PartialEq, Eq, Clone, Hash, Ord, PartialOrd)]
pub struct LocationAndAliasesPairInternal {
    pub(in super::super::location_table) unhandled_geoaddr: UnhandledGeoaddr,
    pub(in super::super::location_table) courses: Vec<Course>,
    pub(in super::super::location_table) aliases: Vec<String>,
}
impl LocationAndAliasesPairInternal {
    #[inline]
    pub fn new(
        unhandled_geoaddr: UnhandledGeoaddr,
        courses: Vec<Course>,
        aliases: Vec<String>,
    ) -> Self {
        Self {
            unhandled_geoaddr,
            courses,
            aliases,
        }
    }
    // pub fn unhandled_geoaddr(&self) -> &UnhandledGeoaddr {
    //     &self.unhandled_geoaddr
    // }
    pub fn courses(&self) -> &Vec<Course> {
        &self.courses
    }
    // pub fn aliases(&self) -> &Vec<String> {
    //     &self.aliases
    // }
}
impl TryFromWithContext<&str> for LocationAndAliasesPairInternal {
    type Err = StoreError;
    type Context<'cxt> = ();

    fn try_from<'cxt, Cxt: Borrow<Self::Context<'cxt>>>(
        s: &str,
        _: Cxt,
    ) -> Result<Self, <Self as TryFromWithContext<&str>>::Err> {
        let data: Vec<&str> = s.split('$').collect();
        if data.len() <= 2 {
            Err(StoreError::ParseError(
                "格式应为 `课程号,班级号/...$地址,经度,纬度,海拔$别名/...`".to_string(),
            ))?
        }

        let mut courses: Vec<Course> = data[0]
            .split('/')
            .map(|s| {
                s.trim().parse::<Course>().unwrap_or_else(|e| {
                    warn!("课程号解析失败，回退为 `invalid`! 错误信息：{e}.");
                    Course::global_course()
                })
            })
            .collect();
        if courses.is_empty() {
            warn!("课程号为空，回退为 `invalid`! ");
            courses.push(Course::global_course());
        }
        data[1]
            .parse::<UnhandledGeoaddr>()
            .map(|location| {
                let aliases: Vec<_> = data
                    .get(2)
                    .into_iter()
                    .map(|data| data.split('/'))
                    .flat_map(|a| a.map(|s| s.trim().to_string()))
                    .collect();
                LocationAndAliasesPairInternal {
                    unhandled_geoaddr: location,
                    aliases,
                    courses,
                }
            })
            .map_err(|e| StoreError::ParseError(format!("位置解析出错：{e}.")))
    }
}

impl Display for LocationAndAliasesPairInternal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let aliases_contents = self.aliases.join("/");
        let courses_contents = if let Some(first) = self.courses().first() {
            let mut first = ToString::to_string(first);
            let rest = self.courses().iter().skip(1).map(ToString::to_string).fold(
                String::new(),
                |mut a, b| {
                    a.push('/');
                    a.push_str(&b);
                    a
                },
            );
            first.push_str(&rest);
            first
        } else {
            String::new()
        };
        debug!("{:?}", self.aliases);
        write!(
            f,
            "{}${}${}",
            courses_contents, self.unhandled_geoaddr, aliases_contents
        )
    }
}
