use std::fmt::{Display, Formatter};
use yapt::impl_point2d;

#[inline]
pub fn triple_point_to_string(result: crate::TriplePoint<u32>) -> String {
    let points = Point3::from((result.0, result.1, result.2));
    log::debug!("本地滑块结果：{points}");
    points.to_string()
}
#[derive(Debug)]
pub struct Point<T>(T, T);
impl_point2d!(Point<T>, Tuple, Tuple);
impl<T: Display> Display for Point<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "%7B%22x%22%3A{}%2C%22y%22%3A{}%7D", self.0, self.1)
    }
}
#[derive(Debug)]
pub struct Point3<T>(Point<T>, Point<T>, Point<T>);
impl<T: Display> Display for Point3<T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "%5B{}%2C{}%2C{}%5D", self.0, self.1, self.2)
    }
}
impl<T> From<(Point<T>, Point<T>, Point<T>)> for Point3<T> {
    #[inline]
    fn from(value: (Point<T>, Point<T>, Point<T>)) -> Self {
        Self(value.0, value.1, value.2)
    }
}
impl<T>
    From<(
        yapt::point_2d::Point<T>,
        yapt::point_2d::Point<T>,
        yapt::point_2d::Point<T>,
    )> for Point3<T>
{
    #[inline]
    fn from(
        value: (
            yapt::point_2d::Point<T>,
            yapt::point_2d::Point<T>,
            yapt::point_2d::Point<T>,
        ),
    ) -> Self {
        Self(
            value.0.into_point_2d(),
            value.1.into_point_2d(),
            value.2.into_point_2d(),
        )
    }
}
#[cfg(test)]
#[test]
fn test() {
    assert_eq!(
        Point3(Point(61, 77), Point(128, 94), Point(210, 74)).to_string(),
        "%5B%7B%22x%22%3A61%2C%22y%22%3A77%7D%2C%7B%22x%22%3A128%2C%22y%22%3A94%7D%2C%7B%22x%22%3A210%2C%22y%22%3A74%7D%5D"
    )
}
