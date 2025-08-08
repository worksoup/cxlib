use bincode::{Decode, Encode};
use getset2::Getset2;
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;

use crate::__private::UnhandledGeoaddr;

/// [`LocationPreprocessorTrait`]
/// 用来对位置作预处理。该特型试图解决如下问题：
///
/// 教师设置签到的位置的“可视地址”并非该位置的真实名称。
/// 例如，程序获取到的名称为“青-692”，但实际签到时，该地点应为“中国浙江省杭州市西湖区柑普洱街太平南路”。
///
/// 可以借助该特型的[`do_preprocess`](LocationPreprocessorTrait::do_preprocess)方法，对该位置进行一定地处理，使之更符合真实情况。
pub trait LocationPreprocessorTrait: Send + Sync {
    /// 方法，消费一个 [`Location`], 生产一个新的 [`Location`].
    /// 默认直接返回入参。
    ///
    /// 注意，该函数不会作用于 [`FromStr`] 和与 `[String; 4]` 间的转换当中。
    #[inline]
    fn do_preprocess(&self, location: Geoaddr) -> Geoaddr {
        location
    }
}
impl LocationPreprocessorTrait for ref_wrapper::Unit {}
pub mod __private {
    use std::{
        borrow::{Borrow, BorrowMut},
        str::FromStr,
    };

    use getset2::Getset2;
    use serde::{Deserialize, Serialize};

    use crate::{Geoaddr, Geolocation, LocationPreprocessorTrait};

    /// # [`UnhandledLocation`]
    /// 没有经过 [`LocationPreprocessorTrait`] 处理过的签到位置。
    ///
    /// 不应使用。
    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        bincode::Decode,
        bincode::Encode,
        Serialize,
        Deserialize,
        Getset2,
    )]
    #[getset2(get_ref(pub), get_mut(pub), set(pub))]
    pub struct UnhandledGeoaddr {
        #[serde(rename = "address")]
        pub(super) unhandled_place_name: String,
        #[serde(flatten)]
        pub(super) geolocation: Geolocation,
    }
    impl UnhandledGeoaddr {
        #[inline(always)]
        pub fn new(unhandled_place_name: String, geolocation: Geolocation) -> Self {
            Self {
                unhandled_place_name,
                geolocation,
            }
        }
        #[inline(always)]
        pub fn get_geolocation(self) -> Geolocation {
            self.geolocation
        }
        #[inline]
        pub fn to_location(self, preprocessor: &impl LocationPreprocessorTrait) -> Geoaddr {
            let Self {
                unhandled_place_name,
                geolocation: location,
            } = self;
            Geoaddr::new(unhandled_place_name, location, preprocessor)
        }
    }
    impl Borrow<Geolocation> for UnhandledGeoaddr {
        #[inline]
        fn borrow(&self) -> &Geolocation {
            &self.geolocation
        }
    }
    impl Borrow<Geolocation> for &UnhandledGeoaddr {
        #[inline]
        fn borrow(&self) -> &Geolocation {
            &self.geolocation
        }
    }
    impl Borrow<Geolocation> for &mut UnhandledGeoaddr {
        #[inline]
        fn borrow(&self) -> &Geolocation {
            &self.geolocation
        }
    }
    impl BorrowMut<Geolocation> for UnhandledGeoaddr {
        #[inline]
        fn borrow_mut(&mut self) -> &mut Geolocation {
            &mut self.geolocation
        }
    }
    impl BorrowMut<Geolocation> for &mut UnhandledGeoaddr {
        #[inline]
        fn borrow_mut(&mut self) -> &mut Geolocation {
            &mut self.geolocation
        }
    }
    impl AsRef<Geolocation> for UnhandledGeoaddr {
        #[inline]
        fn as_ref(&self) -> &Geolocation {
            &self.geolocation
        }
    }
    impl AsMut<Geolocation> for UnhandledGeoaddr {
        #[inline]
        fn as_mut(&mut self) -> &mut Geolocation {
            &mut self.geolocation
        }
    }
    impl From<Geoaddr> for UnhandledGeoaddr {
        #[inline]
        fn from(value: Geoaddr) -> Self {
            let Geoaddr {
                place_name,
                geolocation: location,
            } = value;
            Self {
                unhandled_place_name: place_name,
                geolocation: location,
            }
        }
    }
    impl std::fmt::Display for UnhandledGeoaddr {
        #[inline]
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{},{},{},{}",
                self.unhandled_place_name,
                self.geolocation.lon,
                self.geolocation.lat,
                self.geolocation.alt
            )
        }
    }
    impl FromStr for UnhandledGeoaddr {
        type Err = String;

        #[inline]
        fn from_str(location_str: &str) -> Result<Self, Self::Err> {
            let err = || "位置信息格式错误！格式为：`地址,经度,纬度,海拔`.".to_owned();
            let (unhandled_place_name, location_str) =
                location_str.split_once(',').ok_or_else(err)?;
            let unhandled_place_name = unhandled_place_name.to_owned();
            let geolocation = location_str.parse()?;
            Ok(Self {
                unhandled_place_name,
                geolocation,
            })
        }
    }
}
fn default_alt() -> String {
    "1108".to_owned()
}
/// # [`GeoLocation`]
/// 地理坐标，由经纬度、海拔高度组成。
#[derive(
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Hash,
    Decode,
    Encode,
    Serialize,
    Deserialize,
    Getset2,
)]
#[getset2(get_ref(pub), get_mut(pub), set(pub))]
pub struct Geolocation {
    #[serde(rename = "longitude")]
    lon: String,
    #[serde(rename = "latitude")]
    lat: String,
    #[serde(skip, default = "default_alt")]
    alt: String,
}
impl Geolocation {
    /// Eq to `Self::from_owned_fields([const { String::new() }; 4])`.
    ///
    /// 注意，该函数不会对 [`Location`] 进行预处理。
    #[inline]
    pub fn get_none_location() -> Self {
        Self {
            lon: String::new(),
            lat: String::new(),
            alt: String::new(),
        }
    }
}
impl std::str::FromStr for Geolocation {
    type Err = String;

    #[inline]
    fn from_str(location_str: &str) -> Result<Self, Self::Err> {
        let location_str = location_str.split(',').map(|s| s.trim().to_owned());
        let mut location_str = location_str.into_iter();
        let err = || "位置信息格式错误！格式为：`经度,纬度,海拔`.".to_owned();
        let lon = location_str.next().ok_or_else(err)?;
        let lat = location_str.next().ok_or_else(err)?;
        let alt = location_str.next().ok_or_else(err)?;
        Ok(Self { lon, lat, alt })
    }
}
/// # [`GeoAddr`]
/// 签到位置，由显示地址（一般可在签到完成界面查看）与地理坐标（经纬度与海拔）组成。
#[derive(Debug, Clone)]
pub struct Geoaddr {
    place_name: String,
    geolocation: Geolocation,
}

impl Geoaddr {
    /// Eq to `Self::from_owned_fields([const { String::new() }; 4])`.
    ///
    /// 注意，该函数不会对 [`Location`] 进行预处理。
    #[inline]
    pub fn get_none_location() -> Self {
        Self {
            place_name: String::new(),
            geolocation: Geolocation::get_none_location(),
        }
    }
    #[inline]
    pub fn decompose(self) -> (String, Geolocation) {
        (self.place_name, self.geolocation)
    }
    #[inline]
    pub fn decompose_as(&self) -> (&String, &Geolocation) {
        (&self.place_name, &self.geolocation)
    }
    /// 构造函数，顺序为显示地址、经度、纬度、海拔（单位应该为米，在本程序中该字段无实际用途，可随意）。
    #[inline]
    pub fn new(
        unhandled_place_name: String,
        location: Geolocation,
        preprocessor: &impl LocationPreprocessorTrait,
    ) -> Geoaddr {
        let location = Geoaddr {
            place_name: unhandled_place_name,
            geolocation: location,
        };
        preprocessor.do_preprocess(location)
    }
    /// 地址。
    #[inline]
    pub fn addr(&self) -> &String {
        &self.place_name
    }
    /// 坐标。
    #[inline]
    pub fn location(&self) -> &Geolocation {
        &self.geolocation
    }
    /// 地址。
    #[inline]
    pub fn set_addr(&mut self, addr: String) {
        self.place_name = addr
    }
    /// 坐标。
    #[inline]
    pub fn set_location(&mut self, location: Geolocation) {
        self.geolocation = location
    }
    /// 地址。
    #[inline]
    pub fn get_addr_mut(&mut self) -> &mut String {
        &mut self.place_name
    }
    /// 坐标。
    #[inline]
    pub fn get_location_mut(&mut self) -> &mut Geolocation {
        &mut self.geolocation
    }
}

impl std::fmt::Display for Geoaddr {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().fmt(f)
    }
}
impl AsRef<__private::UnhandledGeoaddr> for Geoaddr {
    fn as_ref(&self) -> &__private::UnhandledGeoaddr {
        unsafe { std::mem::transmute(self) }
    }
}
impl Borrow<__private::UnhandledGeoaddr> for Geoaddr {
    fn borrow(&self) -> &__private::UnhandledGeoaddr {
        self.as_ref()
    }
}
impl Borrow<__private::UnhandledGeoaddr> for &Geoaddr {
    fn borrow(&self) -> &__private::UnhandledGeoaddr {
        self.as_ref()
    }
}

// pub fn 为数据库添加位置(
//     db: &super::sql::DataBase, course_id: i64, 位置: &Location
// ) -> i64 {
//     // 为指定课程添加位置。
//     let mut 位置id = 0_i64;
//     loop {
//         if db.是否存在为某id的位置(位置id) {
//             位置id += 1;
//             continue;
//         }
//         db.添加位置_失败后则(位置id, course_id, 位置, |_, _, _, _| {});
//         break;
//     }
//     位置id
// }
/// #[`LocationWithRange`]
/// 带范围的签到位置。参见 [`Location`], 包含额外的签到范围（半径，单位为米），但不包含海拔信息。
///
/// 使用 [`to_shifted_location`](LocationWithRange::to_shifted_location) 转换为偏移后的 `Location`.
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone, Serialize, Deserialize, Getset2)]
pub struct UnhandledGeoAddrWithRange {
    #[serde(flatten)]
    #[getset2(get_ref(pub), get_mut(pub), set(pub))]
    unhandled_geoaddr: UnhandledGeoaddr,
    #[serde(rename = "locationrange")]
    #[getset2(get_copy(pub), get_mut(pub), set(pub))]
    range: u32,
}

impl UnhandledGeoAddrWithRange {
    #[inline(always)]
    pub fn get_unhandled_geoaddr(self) -> UnhandledGeoaddr {
        self.unhandled_geoaddr
    }
    #[inline]
    pub fn new(addr: String, lon: String, lat: String, range: u32) -> Self {
        Self {
            unhandled_geoaddr: UnhandledGeoaddr {
                unhandled_place_name: addr,
                geolocation: Geolocation {
                    lon,
                    lat,
                    alt: default_alt(),
                },
            },
            range,
        }
    }
    pub fn find_in_html(html: &str) -> Option<UnhandledGeoAddrWithRange> {
        let p = [
            "id=\"locationText\"",
            "id=\"locationLongitude\"",
            "id=\"locationLatitude\"",
            "id=\"locationRange\"",
        ];
        let mut start = [None, None, None, None];
        let mut results1 = Vec::new();
        for i in 0..4 {
            let s = html.find(p[i]);
            start[i] = s;
            if let Some(s) = s {
                let r = &html[s + p[i].len()..html.len()];
                results1.push(r);
            } else {
                return None;
            }
        }
        let mut results2 = Vec::new();
        for r in &results1 {
            let s = r.find("value=\"");
            if let Some(s) = s {
                let r = &r[s + 7..r.len()];
                results2.push(r);
            } else {
                return None;
            }
        }
        let mut results3 = Vec::new();
        for r in &results2 {
            let e = r.find('"');
            if let Some(e) = e {
                let r = &r[0..e];
                results3.push(r);
            } else {
                return None;
            }
        }
        Some(UnhandledGeoAddrWithRange::new(
            results3[0].to_owned(),
            results3[1].to_owned(),
            results3[2].to_owned(),
            if let Ok(s) = results3[3].trim_end_matches('米').parse() {
                s
            } else {
                return None;
            },
        ))
    }
}
