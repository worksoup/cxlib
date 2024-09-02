use image::{
    DynamicImage, GenericImage, GrayImage, ImageBuffer, Luma, LumaA, Pixel, Primitive, Rgba,
};
use imageproc::definitions::Image;
use imageproc::map::{map_colors, map_colors2};
pub use imageproc::point::Point;
use num_traits::NumCast;
use std::ops::Deref;

pub fn get_rect_contains_vertex<T: Primitive, V: Iterator<Item = Point<T>>>(
    vertex: V,
) -> (Point<T>, Point<T>) {
    let mut x_max = T::min_value();
    let mut x_min = T::max_value();
    let mut y_max = T::min_value();
    let mut y_min = T::max_value();
    for p in vertex {
        if p.x > x_max {
            x_max = p.x
        }
        if p.y > y_max {
            y_max = p.y
        }
        if p.x < x_min {
            x_min = p.x
        }
        if p.y < y_min {
            y_min = p.y
        }
    }
    let lt = {
        let x = x_min;
        let y = y_min;
        Point { x, y }
    };
    let rb = {
        let x = x_max;
        let y = y_max;
        Point { x, y }
    };
    (lt, rb)
}

pub fn cut_picture(
    picture: &image::DynamicImage,
    top_left: Point<u32>,
    wh: Point<u32>,
) -> image::DynamicImage {
    picture.crop_imm(top_left.x, top_left.y, wh.x, wh.y)
}

pub fn find_contour_rects<T: Primitive + Eq>(img: &GrayImage) -> Vec<(Point<T>, Point<T>)> {
    let contours = imageproc::contours::find_contours::<T>(img);
    contours
        .into_iter()
        .map(|c| get_rect_contains_vertex(c.points.into_iter()))
        .collect()
}

pub fn image_mean(image: &GrayImage) -> f32 {
    let sum = image
        .pixels()
        .fold((0_f32, 0_usize), |acc, p| (acc.0 + p[0] as f32, acc.1 + 1));
    sum.0 / sum.1 as f32
}

pub fn image_sum<P: Primitive, Container>(image: &ImageBuffer<Luma<P>, Container>) -> f32
where
    Container: Deref<Target = [<Luma<P> as Pixel>::Subpixel]>,
{
    let sum = image
        .pixels()
        .fold(0_f32, |acc, p| acc + <f32 as NumCast>::from(p[0]).unwrap());
    sum
}

pub fn rgb_alpha_channel<I, C>(image: &I) -> Image<Luma<C>>
where
    I: GenericImage<Pixel = Rgba<C>>,
    Rgba<C>: Pixel<Subpixel = C>,
    C: Primitive,
{
    map_colors(image, |p| Luma([p[3]]))
}

pub fn luma_alpha_channel<I, C>(image: &I) -> Image<Luma<C>>
where
    I: GenericImage<Pixel = LumaA<C>>,
    LumaA<C>: Pixel<Subpixel = C>,
    C: Primitive,
{
    map_colors(image, |p| Luma([p[1]]))
}

pub fn download_image(
    agent: &ureq::Agent,
    image_url: &str,
) -> Result<DynamicImage, Box<ureq::Error>> {
    let mut v = Vec::new();
    agent
        .get(image_url)
        .call()?
        .into_reader()
        .read_to_end(&mut v)
        .unwrap();
    let img = image::ImageReader::new(std::io::Cursor::new(v))
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap();
    Ok(img)
}

pub fn find_sub_image(big_image: &DynamicImage, small_image: &DynamicImage) -> u32 {
    let small_image_alpha = crate::rgb_alpha_channel(small_image);
    let rects = crate::find_contour_rects::<u32>(&small_image_alpha);
    let (lt, rb) = rects[0];
    let small_image = crate::cut_picture(small_image, lt, rb - lt);
    let small_image = small_image.to_luma8();
    let mean = image_mean(&small_image);
    let small_image = map_colors(&small_image, |p| Luma([p[0] as f32 - mean]));
    let mut max_ncc = 0.0;
    let mut max_x = 0;
    let small_w = small_image.width();
    let big_w = big_image.width();
    let big_img = crate::cut_picture(
        big_image,
        lt,
        Point {
            x: big_w - small_w,
            y: 0,
        } + (rb - lt),
    );
    let big_img = big_img.to_luma8();
    let big_img = DynamicImage::from(big_img);
    for x in 0..big_img.width() - small_image.width() {
        let window = crate::cut_picture(
            &big_img,
            Point { x, y: 0 },
            Point {
                x: small_image.width(),
                y: small_image.height(),
            },
        )
        .to_luma8();
        let window_mean = image_mean(&window);
        let window = map_colors(&window, |p| Luma([p[0] as f32 - window_mean]));
        let a = map_colors2(&window, &small_image, |w, t| Luma([w[0] * t[0]]));
        let b = map_colors(&window, |w| Luma([w[0] * w[0]]));
        let ncc = image_sum(&a) / image_sum(&b);
        if ncc > max_ncc {
            max_x = x;
            max_ncc = ncc;
        }
    }
    max_x
}
