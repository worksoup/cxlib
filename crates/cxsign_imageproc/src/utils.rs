use image::buffer::ConvertBuffer;
use image::{
    DynamicImage, GenericImage, GenericImageView, GrayImage, Luma, LumaA, Pixel, Primitive, Rgba,
    SubImage,
};
use imageproc::definitions::Image;
use imageproc::map::{map_colors, map_colors2};
pub use imageproc::point::Point;
use num_traits::ToPrimitive;
use std::ops::Add;

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

pub fn cut_picture<I: GenericImageView>(
    picture: &I,
    top_left: Point<u32>,
    wh: Point<u32>,
) -> SubImage<&I> {
    image::imageops::crop_imm(picture, top_left.x, top_left.y, wh.x, wh.y)
}

pub fn find_contour_rects<T: Primitive + Eq>(img: &GrayImage) -> Vec<(Point<T>, Point<T>)> {
    let contours = imageproc::contours::find_contours::<T>(img);
    contours
        .into_iter()
        .map(|c| get_rect_contains_vertex(c.points.into_iter()))
        .collect()
}

pub fn image_sum<Pixel: image::Pixel, Image: GenericImageView<Pixel = Pixel>>(
    image: &Image,
) -> Vec<f64> {
    let zero = vec![0_f64; Pixel::CHANNEL_COUNT as usize];
    let sum = image.pixels().fold(zero, |mut acc, p| {
        let channels = p.2.channels();
        for (index, acc) in acc.iter_mut().enumerate() {
            *acc = acc.add(channels[index].to_f64().expect("Can't convert to f64"));
        }
        acc
    });
    sum
}
pub fn image_mean<Pixel: image::Pixel, Image: GenericImageView<Pixel = Pixel>>(
    image: &Image,
) -> Vec<f64> {
    let sum = image_sum(image);
    let (w, h) = image.dimensions();
    let size = (w * h) as f64;
    sum.into_iter().map(|sum| sum / size).collect()
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

pub fn image_from_bytes(bytes: Vec<u8>) -> DynamicImage {
    image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap()
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
    let img = image_from_bytes(v);
    Ok(img)
}
pub fn find_max_ncc(big_img: &GrayImage, small_image: &GrayImage) -> u32 {
    let mean = image_mean(small_image);
    let small_image = map_colors(small_image, |p| Luma([p[0] as f64 - mean[0]]));
    let mut max_ncc = 0.0;
    let mut max_x = 0;
    for x in 0..big_img.width() - small_image.width() {
        let window = crate::cut_picture(
            big_img,
            Point { x, y: 0 },
            Point {
                x: small_image.width(),
                y: small_image.height(),
            },
        )
        .to_image();
        let window_mean = image_mean(&window);
        let window = map_colors(&window, |p| Luma([p[0] as f64 - window_mean[0]]));
        let a = map_colors2(&window, &small_image, |w, t| Luma([w[0] * t[0]]));
        let b = map_colors(&window, |w| Luma([w[0] * w[0]]));
        let ncc = image_sum(&a)[0] / image_sum(&b)[0];
        if ncc > max_ncc {
            max_x = x;
            max_ncc = ncc;
        }
    }
    max_x
}
pub fn find_sub_image<F: Fn(&GrayImage, &GrayImage) -> u32>(
    big_image: &DynamicImage,
    small_image: &DynamicImage,
    a: F,
) -> u32 {
    let small_image_alpha = rgb_alpha_channel(small_image);
    let rects = crate::find_contour_rects::<u32>(&small_image_alpha);
    let (lt, rb) = rects[0];
    let small_image = cut_picture(small_image, lt, rb - lt).to_image();
    let small_image = small_image.convert();
    let small_w = small_image.width();
    let big_w = big_image.width();
    let big_img = cut_picture(
        big_image,
        lt,
        Point {
            x: big_w - small_w,
            y: 0,
        } + (rb - lt),
    );
    let big_img = big_img.to_image().convert();
    a(&big_img, &small_image)
}
