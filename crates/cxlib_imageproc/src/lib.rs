pub mod map;

use crate::map::map_colors;
use image::buffer::ConvertBuffer;
use image::{
    DynamicImage, GenericImage, GenericImageView, GrayImage, ImageBuffer, ImageError, Luma, LumaA,
    Pixel, Primitive, Rgba, SubImage,
};
use imageproc::contours::find_contours;
use num_traits::ToPrimitive;
use std::ops::Add;
use std::path::Path;
pub use yapt::point_2d::Point;
use yapt::point_2d::Point2D;

pub type Image<P> = ImageBuffer<P, Vec<<P as Pixel>::Subpixel>>;
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

pub fn image_sum<Pixel: image::Pixel, Image: GenericImageView<Pixel = Pixel>>(
    image: &Image,
    mask: &[bool],
) -> (Vec<f64>, usize) {
    fn add<P: image::Pixel>(
        (mut acc, count): (Vec<f64>, usize),
        p: (usize, (u32, u32, P)),
    ) -> (Vec<f64>, usize) {
        let channels = p.1 .2.channels();
        for (index, acc) in acc.iter_mut().enumerate() {
            *acc = acc.add(channels[index].to_f64().expect("Can't convert to f64"));
        }
        (acc, count + 1)
    }
    let zero = vec![0_f64; Pixel::CHANNEL_COUNT as usize];
    let sum =
        if mask.len() < (image.width() * image.width()) as usize {
            image
                .pixels()
                .enumerate()
                .fold((zero, 0_usize), |acc, p| add(acc, p))
        } else {
            image.pixels().enumerate().fold((zero, 0_usize), |acc, p| {
                if mask[p.0] {
                    add(acc, p)
                } else {
                    acc
                }
            })
        };
    sum
}
pub fn image_mean<Pixel: image::Pixel, Image: GenericImageView<Pixel = Pixel>>(
    image: &Image,
    mask: &[bool],
) -> Vec<f64> {
    let (sum, count) = image_sum(image, mask);
    sum.into_iter().map(|sum| sum / count as f64).collect()
}

pub fn rgb_alpha_channel<I, C>(image: &I) -> Image<Luma<C>>
where
    I: GenericImage<Pixel = Rgba<C>>,
    Rgba<C>: Pixel<Subpixel = C>,
    C: Primitive,
{
    map_colors(image, |_, _, p| Luma([p[3]]))
}

pub fn luma_alpha_channel<I, C>(image: &I) -> Image<Luma<C>>
where
    I: GenericImage<Pixel = LumaA<C>>,
    LumaA<C>: Pixel<Subpixel = C>,
    C: Primitive,
{
    map_colors(image, |_, _, p| Luma([p[1]]))
}

pub fn image_from_bytes(bytes: Vec<u8>) -> DynamicImage {
    image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap()
}

pub fn open_image<P: AsRef<Path>>(p: P) -> Result<DynamicImage, ImageError> {
    image::ImageReader::open(p)?.with_guessed_format()?.decode()
}
pub mod match_template {
    use image::GrayImage;
    pub use imageproc::template_matching::MatchTemplateMethod;
    use imageproc::template_matching::{
        find_extremes, match_template_parallel, match_template_with_mask_parallel, Extremes,
    };
    fn extremes_to_result(extremes: Extremes<f32>, method: MatchTemplateMethod) -> u32 {
        match method {
            MatchTemplateMethod::SumOfSquaredErrors
            | MatchTemplateMethod::SumOfSquaredErrorsNormalized => extremes.min_value_location.0,
            MatchTemplateMethod::CrossCorrelation
            | MatchTemplateMethod::CrossCorrelationNormalized => extremes.max_value_location.0,
        }
    }
    pub fn match_template_for_slide(
        big_image: &GrayImage,
        small_image: &GrayImage,
        method: MatchTemplateMethod,
        mask: &GrayImage,
    ) -> u32 {
        let image = match_template_with_mask_parallel(big_image, small_image, method, mask);
        let extremes = find_extremes(&image);
        extremes_to_result(extremes, method)
    }
    pub fn match_template_for_rotate(
        big_image: &GrayImage,
        small_image: &GrayImage,
        method: MatchTemplateMethod,
    ) -> u32 {
        let image = match_template_parallel(big_image, small_image, method);
        let extremes = find_extremes(&image);
        extremes_to_result(extremes, method)
    }
}
fn find_contour_rects<T: Primitive + Eq>(img: &GrayImage) -> Vec<(Point<T>, Point<T>)> {
    let contours = find_contours::<T>(img);
    contours
        .into_iter()
        .map(|c| get_rect_contains_vertex(c.points.into_iter().map(Point2D::into_point)))
        .collect()
}
pub fn find_sub_image<F: Fn(&GrayImage, &GrayImage, &GrayImage) -> u32>(
    big_image: &DynamicImage,
    small_image: &DynamicImage,
    a: F,
) -> u32 {
    let small_image_alpha = rgb_alpha_channel(small_image);
    let rects = find_contour_rects::<u32>(&small_image_alpha);
    let (lt, rb) = rects[0];
    let small_image = cut_picture(small_image, lt, rb - lt);
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
    let small_image = small_image.to_image();
    let big_image = big_img.to_image().convert();
    let mask = rgb_alpha_channel(&small_image);
    a(&big_image, &small_image.convert(), &mask)
}
pub mod click_captcha_utils {
    use crate::cut_picture;
    use crate::map::map_colors;
    use image::{DynamicImage, GrayImage, Luma, Primitive};
    use yapt::point_2d::Point2D;

    pub fn find_icon(image: &DynamicImage) -> GrayImage {
        let image = cut_picture(image, (0, 0).into_point(), (320, 160).into_point());
        let image = image.to_image();
        map_colors(&image, |_, _, p| {
            let [r, g, b, _a] = p.0;
            fn sq<T: Primitive>(t: T) -> T {
                t * t
            }
            const E: i32 = 7;
            const M: i32 = 3 * E * E - E + 1015;
            const B: i32 = E;
            const W: i32 = 255 - E;
            let [r, g, b] = [r as i32, g as i32, b as i32];
            let m = (r + g + b) / 3;
            let gray = sq(r - m) + sq(g - m) + sq(b - m) < M;
            let black = r + g + b < 3 * B;
            let white = r + g + b > 3 * W;
            if gray && (white || black) {
                Luma::from([if white { 255 } else { 0 }])
            } else {
                Luma::from([127])
            }
        })
    }
}
pub mod rotate_captcha_utils {
    use crate::{map::map_colors2_parallel, rgb_alpha_channel};
    use image::{
        buffer::ConvertBuffer, DynamicImage, GenericImage, GenericImageView, GrayImage,
        ImageBuffer, Luma, Rgba,
    };
    use std::f64::consts::PI;
    use std::sync::{Arc, Mutex};
    use yapt::point_2d::{Point, Point2D};
    pub fn get_edge<const SPLIT_COUNT: u32>(
        outer: &DynamicImage,
        mask: &GrayImage,
    ) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let pixels = Arc::new(Mutex::new(vec![
            (0u16, 0u16, 0u16, 0u16, 0u8);
            SPLIT_COUNT as usize
        ]));
        let mask_wh = mask.dimensions().into_point();
        let get_masked_image = |x, y, Luma([p]), Rgba([r, g, b, a])| {
            if p > 0 {
                let (ax, ay) = (x as i32, y as i32);
                let Point { x: w, y: h } = mask_wh;
                let (w, h) = (w as i32, h as i32);
                let Point { x: ax, y: ay } = (ax, ay).into_point() - (w / 2, h / 2).into_point();
                let angle = if ax == 0 {
                    if ay > 0 {
                        PI / 2.0
                    } else {
                        -PI / 2.0
                    }
                } else {
                    let tg = (ay as f64) / (ax as f64);
                    if ax > 0 {
                        tg.atan()
                    } else {
                        tg.atan() + PI
                    }
                };
                let s = angle / PI * SPLIT_COUNT as f64 / 2.0;
                let s = (s as isize + SPLIT_COUNT as isize / 4) as usize;
                let mut pixels = pixels.lock().unwrap();
                pixels[s].0 += r as u16;
                pixels[s].1 += g as u16;
                pixels[s].2 += b as u16;
                pixels[s].3 += a as u16;
                pixels[s].4 += 1;
                Luma([0u8])
            } else {
                Luma([0u8])
            }
        };
        let flat_map = |(r, g, b, a, c): &(u16, u16, u16, u16, u8)| {
            let (r, g, b, a) = (*r as f32, *g as f32, *b as f32, *a as f32);
            let c = *c as f32;
            let (r, g, b, a) = (r / c, g / c, b / c, a);
            [r as u8, g as u8, b as u8, a as u8]
        };
        let _ = map_colors2_parallel(mask, outer, get_masked_image);
        let pixels_outer = pixels
            .lock()
            .unwrap()
            .iter()
            .flat_map(flat_map)
            .collect::<Vec<_>>();
        ImageBuffer::from_vec(1, SPLIT_COUNT, pixels_outer).unwrap()
    }

    pub fn match_angle<F: Fn(&GrayImage, &GrayImage) -> u32>(
        outer: &DynamicImage,
        inner: &DynamicImage,
        matcher: F,
    ) -> u32 {
        let outer_mask = rgb_alpha_channel(outer);
        let inner_mask = rgb_alpha_channel(inner);
        let mask = map_colors2_parallel(&outer_mask, &inner_mask, |_, _, p, q| {
            Luma([p.0[0] & q.0[0]])
        });
        const SPLIT_COUNT: u32 = 360;
        let outer_edge = get_edge::<SPLIT_COUNT>(outer, &mask);
        let inner_edge = get_edge::<SPLIT_COUNT>(inner, &mask);
        let wh = inner_edge.height();
        let mut inner = ImageBuffer::new(wh, wh);
        for y in 0..wh {
            for x in 0..wh {
                unsafe {
                    let pix = inner_edge.unsafe_get_pixel(0, (wh + y - x) % wh);
                    inner.unsafe_put_pixel(x, y, pix);
                }
            }
        }
        matcher(&inner.convert(), &outer_edge.convert()) * 200 / SPLIT_COUNT
    }
}
