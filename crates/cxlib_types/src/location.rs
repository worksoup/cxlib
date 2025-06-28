use cxlib_base_types::{
    __private::UnhandledGeoaddr, Geoaddr, Geolocation, LocationPreprocessorTrait,
};
use rand::Rng;
use serde::{Deserialize, Serialize};

/// #[`LocationWithRange`]
/// 带范围的签到位置。参见 [`Location`], 包含额外的签到范围（半径，单位为米），但不包含海拔信息。
///
/// 使用 [`to_shifted_location`](LocationWithRange::to_shifted_location) 转换为偏移后的 `Location`.
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct LocationWithRange {
    #[serde(rename = "address")]
    addr: String,
    #[serde(rename = "longitude")]
    lon: String,
    #[serde(rename = "latitude")]
    lat: String,
    #[serde(rename = "locationrange")]
    range: u32,
}

impl LocationWithRange {
    #[inline]
    pub fn new(addr: String, lon: String, lat: String, range: u32) -> Self {
        Self {
            addr,
            lon,
            lat,
            range,
        }
    }
    pub fn find_in_html(html: &str) -> Option<LocationWithRange> {
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
        Some(LocationWithRange {
            addr: results3[0].to_owned(),
            lon: results3[1].to_owned(),
            lat: results3[2].to_owned(),
            range: if let Ok(s) = results3[3].trim_end_matches('米').parse() {
                s
            } else {
                return None;
            },
        })
    }
    /// 本类型的数据一般直接从签到信息内获取，为避免与预设位置完全一致，本程序将默认以随机偏移一定距离后的位置作为签到位置，使之符合真实情况。
    ///
    /// 偏移距离在范围的百分之五以内，即 0--5 米到 0--100 米不等，但绝不会超出范围。
    ///
    /// 由于本类型不包含海拔数据，海拔将被设置为 `1108`(米).
    pub fn into_shifted_unhandled_geoaddr(self) -> UnhandledGeoaddr {
        const R: f64 = 6371393.0;
        let LocationWithRange {
            addr: unhandled_place_name,
            lon,
            lat,
            range,
        } = self;
        let lat: f64 = lat.parse().unwrap();
        let lon: f64 = lon.parse().unwrap();
        let mut rng = rand::rng();
        // r | [0.0..0.05].
        let mut r: f64 = rng.random_range(0.0..0.05);
        use std::f64::consts::{PI, TAU};
        // theta | [0.0..TAU].
        let theta = rng.random_range(0.0..TAU);
        r *= (range as f64)
            / R
            / (1.0 - theta.cos().powi(2) * (lat * PI / 180.0).sin().powi(2)).sqrt();
        let lat = format!("{:.6}", ((lat * PI / 180.0) + r * theta.sin()) / PI * 180.0);
        let lon = format!("{:.6}", (lon * PI / 180.0 + r * theta.cos()) / PI * 180.0);
        UnhandledGeoaddr {
            unhandled_place_name,
            geolocation: Geolocation {
                lon,
                lat,
                alt: "1108".to_owned(),
            },
        }
    }
    /// 本类型的数据一般直接从签到信息内获取，为避免与预设位置完全一致，本程序将默认以随机偏移一定距离后的位置作为签到位置，使之符合真实情况。
    ///
    /// 偏移距离在范围的百分之五以内，即 0--5 米到 0--100 米不等，但绝不会超出范围。
    ///
    /// 由于本类型不包含海拔数据，海拔将被设置为 `1108`(米).
    #[inline]
    pub fn into_shifted_geoaddr(self, preprocessor: &impl LocationPreprocessorTrait) -> Geoaddr {
        self.into_shifted_unhandled_geoaddr()
            .to_location(preprocessor)
    }
    /// 本类型的数据一般直接从签到信息内获取，为避免与预设位置完全一致，本程序将默认以随机偏移一定距离后的位置作为签到位置，使之符合真实情况。
    ///
    /// 偏移距离在范围的百分之五以内，即 0--5 米到 0--100 米不等，但绝不会超出范围。
    ///
    /// 由于本类型不包含海拔数据，海拔将被设置为 `1108`(米).
    #[inline]
    pub fn to_shifted_unhandled_geoaddr(&self) -> UnhandledGeoaddr {
        self.clone().into_shifted_unhandled_geoaddr()
    }
    /// 本类型的数据一般直接从签到信息内获取，为避免与预设位置完全一致，本程序将默认以随机偏移一定距离后的位置作为签到位置，使之符合真实情况。
    ///
    /// 偏移距离在范围的百分之五以内，即 0--5 米到 0--100 米不等，但绝不会超出范围。
    ///
    /// 由于本类型不包含海拔数据，海拔将被设置为 `1108`(米).
    #[inline]
    pub fn to_shifted_geoaddr(&self, preprocessor: &impl LocationPreprocessorTrait) -> Geoaddr {
        self.clone().into_shifted_geoaddr(preprocessor)
    }
    pub fn get_range(&self) -> u32 {
        self.range
    }
}
#[cfg(test)]
mod tests {
    use cxlib_base_types::LocationPreprocessorTrait;

    use crate::LocationWithRange;
    struct DefaultLocationPreprocessor;
    impl LocationPreprocessorTrait for DefaultLocationPreprocessor {}
    #[test]
    fn a() {
        let l = LocationWithRange {
            addr: "addr".into(),
            lon: "108.840053".into(),
            lat: "34.129522".into(),
            range: 100,
        };
        println!("{}", l.to_shifted_geoaddr(&DefaultLocationPreprocessor))
    }
}
