use crate::map::map_colors;
use image::{
    DynamicImage, GenericImage, GenericImageView, GrayImage, ImageBuffer, ImageError, Luma, LumaA,
    Pixel, Primitive, Rgba, SubImage,
};
use imageproc::contours::find_contours;
pub use imageproc::point::Point;
use num_traits::ToPrimitive;
use std::ops::Add;
use std::path::Path;

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

pub mod map {
    use image::{GenericImage, GenericImageView, ImageBuffer, Pixel};
    use imageproc::definitions::Image;
    pub use imageproc::point::Point;
    pub fn map_colors<I, P, Q, F>(image: &I, f: F) -> Image<Q>
    where
        I: GenericImageView<Pixel = P>,
        P: Pixel,
        Q: Pixel,
        F: Fn(P) -> Q,
    {
        let (width, height) = image.dimensions();
        let mut out: ImageBuffer<Q, Vec<Q::Subpixel>> = ImageBuffer::new(width, height);

        for y in 0..height {
            for x in 0..width {
                unsafe {
                    let pix = image.unsafe_get_pixel(x, y);
                    out.unsafe_put_pixel(x, y, f(pix));
                }
            }
        }

        out
    }
    pub fn map_colors2<I, J, P, Q, R, F>(image1: &I, image2: &J, f: F) -> Image<R>
    where
        I: GenericImageView<Pixel = P>,
        J: GenericImageView<Pixel = Q>,
        P: Pixel,
        Q: Pixel,
        R: Pixel,
        F: Fn(P, Q) -> R,
    {
        assert_eq!(image1.dimensions(), image2.dimensions());

        let (width, height) = image1.dimensions();
        let mut out: ImageBuffer<R, Vec<R::Subpixel>> = ImageBuffer::new(width, height);

        for y in 0..height {
            for x in 0..width {
                unsafe {
                    let p = image1.unsafe_get_pixel(x, y);
                    let q = image2.unsafe_get_pixel(x, y);
                    out.unsafe_put_pixel(x, y, f(p, q));
                }
            }
        }

        out
    }
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

pub fn open_image<P: AsRef<Path>>(p: P) -> Result<DynamicImage, ImageError> {
    image::ImageReader::open(p)?.with_guessed_format()?.decode()
}
pub mod slide_solvers {
    use crate::map::{map_colors, map_colors2};
    use crate::{cut_picture, image_mean, image_sum, rgb_alpha_channel};
    use image::{buffer::ConvertBuffer, DynamicImage, GrayImage, Luma, SubImage};
    use imageproc::definitions::Image;
    use imageproc::point::Point;
    use imageproc::template_matching::{
        find_extremes, match_template_with_mask_parallel, MatchTemplateMethod,
    };
    pub fn find_max_ncc(
        big_image: SubImage<&DynamicImage>,
        small_image: SubImage<&DynamicImage>,
    ) -> u32 {
        let mask = rgb_alpha_channel(&small_image.to_image());
        let mask_mean = *image_mean(&mask, &[]).last().expect("No image mean");
        let mask = mask
            .iter()
            .map(|p| (*p as f64) >= mask_mean)
            .collect::<Vec<_>>();
        let mean = image_mean(&*small_image, &mask);
        let small_image: GrayImage = small_image.to_image().convert();
        let small_image = map_colors(&small_image, |p| Luma([p[0] as f64 - mean[0]]));
        let mut max_ncc = 0.0;
        let mut max_x = 0;
        let big_image: GrayImage = big_image.to_image().convert();
        for x in 0..big_image.width() - small_image.width() {
            let window = cut_picture(
                &big_image,
                Point { x, y: 0 },
                Point {
                    x: small_image.width(),
                    y: small_image.height(),
                },
            );
            let window_mean = image_mean(&*window, &mask);
            let window = map_colors(&*window, |p| Luma([p[0] as f64 - window_mean[0]]));
            let a = map_colors2(&window, &small_image, |w, t| Luma([w[0] * t[0]]));
            let b = map_colors(&window, |w| Luma([w[0] * w[0]]));
            let ncc = image_sum(&a, &mask).0[0] / image_sum(&b, &mask).0[0];
            if ncc > max_ncc {
                max_x = x;
                max_ncc = ncc;
            }
        }
        max_x
    }
    /// 目前的最优解。
    pub fn find_min_sum_of_squared_errors(
        big_image: SubImage<&DynamicImage>,
        small_image: SubImage<&DynamicImage>,
    ) -> u32 {
        let image = imageproc_match(
            big_image,
            small_image,
            MatchTemplateMethod::SumOfSquaredErrors,
        );
        find_extremes(&image).min_value_location.0
    }
    pub fn find_min_sum_of_squared_errors_normalized(
        big_image: SubImage<&DynamicImage>,
        small_image: SubImage<&DynamicImage>,
    ) -> u32 {
        let image = imageproc_match(
            big_image,
            small_image,
            MatchTemplateMethod::SumOfSquaredErrorsNormalized,
        );
        find_extremes(&image).min_value_location.0
    }
    pub fn find_max_cross_correlation(
        big_image: SubImage<&DynamicImage>,
        small_image: SubImage<&DynamicImage>,
    ) -> u32 {
        let image = imageproc_match(
            big_image,
            small_image,
            MatchTemplateMethod::CrossCorrelation,
        );
        find_extremes(&image).max_value_location.0
    }
    pub fn find_max_cross_correlation_normalized(
        big_image: SubImage<&DynamicImage>,
        small_image: SubImage<&DynamicImage>,
    ) -> u32 {
        let image = imageproc_match(
            big_image,
            small_image,
            MatchTemplateMethod::CrossCorrelationNormalized,
        );
        find_extremes(&image).max_value_location.0
    }
    pub fn imageproc_match(
        big_image: SubImage<&DynamicImage>,
        small_image: SubImage<&DynamicImage>,
        method: MatchTemplateMethod,
    ) -> Image<Luma<f32>> {
        let big_image: GrayImage = big_image.to_image().convert();
        let small_image = small_image.to_image();
        let mask = rgb_alpha_channel(&small_image);
        let template = small_image.convert();
        match_template_with_mask_parallel(&big_image, &template, method, &mask)
    }
}
fn find_contour_rects<T: Primitive + Eq>(img: &GrayImage) -> Vec<(Point<T>, Point<T>)> {
    let contours = find_contours::<T>(img);
    contours
        .into_iter()
        .map(|c| get_rect_contains_vertex(c.points.into_iter()))
        .collect()
}
pub fn find_sub_image<F: Fn(SubImage<&DynamicImage>, SubImage<&DynamicImage>) -> u32>(
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
    a(big_img, small_image)
}
pub mod click_captcha_utils {
    use crate::cut_picture;
    use crate::map::map_colors;
    use image::{DynamicImage, GrayImage, Luma, Primitive};
    use imageproc::point::Point;

    pub fn find_icon(image: &DynamicImage) -> GrayImage {
        let image = cut_picture(image, Point::new(0, 0), Point::new(320, 160));
        let image = image.to_image();
        map_colors(&image, |p| {
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
