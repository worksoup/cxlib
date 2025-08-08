use cxlib_base_types::{
    __private::UnhandledGeoaddr, Geoaddr, LocationPreprocessorTrait, UnhandledGeoAddrWithRange,
};
use rand::Rng;
pub trait UnhandledGeoAddrWithRangeExt: Sized + Clone {
    /// 本类型的数据一般直接从签到信息内获取，为避免与预设位置完全一致，本程序将默认以随机偏移一定距离后的位置作为签到位置，使之符合真实情况。
    ///
    /// 偏移距离在范围的百分之五以内，即 0--5 米到 0--100 米不等，但绝不会超出范围。
    ///
    /// 由于本类型不包含海拔数据，海拔将被设置为 `1108`(米).
    fn into_shifted_unhandled_geoaddr(self) -> UnhandledGeoaddr;
    /// 本类型的数据一般直接从签到信息内获取，为避免与预设位置完全一致，本程序将默认以随机偏移一定距离后的位置作为签到位置，使之符合真实情况。
    ///
    /// 偏移距离在范围的百分之五以内，即 0--5 米到 0--100 米不等，但绝不会超出范围。
    ///
    /// 由于本类型不包含海拔数据，海拔将被设置为 `1108`(米).
    #[inline]
    fn into_shifted_geoaddr(self, preprocessor: &impl LocationPreprocessorTrait) -> Geoaddr {
        self.into_shifted_unhandled_geoaddr()
            .to_location(preprocessor)
    }
    /// 本类型的数据一般直接从签到信息内获取，为避免与预设位置完全一致，本程序将默认以随机偏移一定距离后的位置作为签到位置，使之符合真实情况。
    ///
    /// 偏移距离在范围的百分之五以内，即 0--5 米到 0--100 米不等，但绝不会超出范围。
    ///
    /// 由于本类型不包含海拔数据，海拔将被设置为 `1108`(米).
    #[inline]
    fn to_shifted_unhandled_geoaddr(&self) -> UnhandledGeoaddr {
        self.clone().into_shifted_unhandled_geoaddr()
    }
    /// 本类型的数据一般直接从签到信息内获取，为避免与预设位置完全一致，本程序将默认以随机偏移一定距离后的位置作为签到位置，使之符合真实情况。
    ///
    /// 偏移距离在范围的百分之五以内，即 0--5 米到 0--100 米不等，但绝不会超出范围。
    ///
    /// 由于本类型不包含海拔数据，海拔将被设置为 `1108`(米).
    #[inline]
    fn to_shifted_geoaddr(&self, preprocessor: &impl LocationPreprocessorTrait) -> Geoaddr {
        self.clone().into_shifted_geoaddr(preprocessor)
    }
}
impl UnhandledGeoAddrWithRangeExt for UnhandledGeoAddrWithRange {
    fn into_shifted_unhandled_geoaddr(self) -> UnhandledGeoaddr {
        const R: f64 = 6371393.0;
        let unhandled_geoaddr = self.unhandled_geoaddr();
        let range = self.range();
        let lat: f64 = unhandled_geoaddr.geolocation().lat().parse().unwrap();
        let lon: f64 = unhandled_geoaddr.geolocation().lon().parse().unwrap();
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
        {
            let mut geoaddr = self.get_unhandled_geoaddr();
            *geoaddr.geolocation_mut().lon_mut() = lon;
            *geoaddr.geolocation_mut().lat_mut() = lat;
            geoaddr
        }
    }
}
#[cfg(test)]
mod tests {
    use cxlib_base_types::LocationPreprocessorTrait;

    use crate::{UnhandledGeoAddrWithRange, UnhandledGeoAddrWithRangeExt};
    struct DefaultLocationPreprocessor;
    impl LocationPreprocessorTrait for DefaultLocationPreprocessor {}
    #[test]
    fn a() {
        let l = UnhandledGeoAddrWithRange::new(
            "addr".into(),
            "108.840053".into(),
            "34.129522".into(),
            100,
        );
        println!("{}", l.to_shifted_geoaddr(&DefaultLocationPreprocessor))
    }
}
