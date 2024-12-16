use image::{GenericImage, GenericImageView, ImageBuffer, Pixel};
use imageproc::definitions::Image;
pub fn map_colors_to<I, J, P, Q, F>(image: &I, out: &mut J, f: F)
where
    I: GenericImageView<Pixel = P>,
    J: GenericImageView<Pixel = Q> + GenericImage,
    P: Pixel,
    Q: Pixel,
    F: Fn(u32, u32, P) -> Q,
{
    let (width, height) = image.dimensions();
    for y in 0..height {
        for x in 0..width {
            unsafe {
                let pix = image.unsafe_get_pixel(x, y);
                out.unsafe_put_pixel(x, y, f(x, y, pix));
            }
        }
    }
}
pub fn map_colors<I, P, Q, F>(image: &I, f: F) -> Image<Q>
where
    I: GenericImageView<Pixel = P>,
    P: Pixel,
    Q: Pixel,
    F: Fn(u32, u32, P) -> Q,
{
    let (width, height) = image.dimensions();
    let mut out: ImageBuffer<Q, Vec<Q::Subpixel>> = ImageBuffer::new(width, height);
    map_colors_to(image, &mut out, f);
    out
}
pub fn map_colors_in_place<I, P, F>(image: &mut I, f: F)
where
    I: GenericImageView<Pixel = P> + GenericImage,
    P: Pixel,
    F: Fn(u32, u32, P) -> P,
{
    let out = image as *mut I;
    unsafe { map_colors_to(image, &mut *out, f) }
}
pub fn map_colors_parallel_to<I, J, P, Q, F>(image: &I, out: &mut J, f: F)
where
    I: GenericImageView<Pixel = P> + Sync,
    J: GenericImageView<Pixel = Q> + Sync + GenericImage,
    P: Pixel,
    Q: Pixel + Send + Sync,
    <Q as Pixel>::Subpixel: Send + Sync,
    F: Fn(u32, u32, P) -> Q + Sync,
{
    assert_eq!(image.dimensions(), out.dimensions());
    use rayon::prelude::*;
    let (width, height) = image.dimensions();
    let map = |out: &mut J| {
        let out = &*out;
        (0..height).into_par_iter().for_each(|y| {
            for x in 0..width {
                unsafe {
                    let pix = image.unsafe_get_pixel(x, y);
                    let out: *const _ = out as *const _;
                    let out: *mut _ = std::mem::transmute::<_, *mut J>(out);
                    (*out).unsafe_put_pixel(x, y, f(x, y, pix));
                }
            }
        })
    };
    map(out);
}
pub fn map_colors_parallel<I, P, Q, F>(image: &I, f: F) -> Image<Q>
where
    I: GenericImageView<Pixel = P> + Sync,
    P: Pixel,
    Q: Pixel + Send + Sync,
    <Q as Pixel>::Subpixel: Send + Sync,
    F: Fn(u32, u32, P) -> Q + Sync,
{
    let (width, height) = image.dimensions();
    let mut out: ImageBuffer<Q, Vec<Q::Subpixel>> = ImageBuffer::new(width, height);
    map_colors_parallel_to(image, &mut out, f);
    out
}
pub fn map_colors_parallel_in_place<I, P, F>(image: &mut I, f: F)
where
    I: GenericImageView<Pixel = P> + GenericImage + Sync,
    P: Pixel + Send + Sync,
    <P as Pixel>::Subpixel: Send + Sync,
    F: Fn(u32, u32, P) -> P + Sync,
{
    let out = image as *mut I;
    unsafe { map_colors_parallel_to(image, &mut *out, f) }
}
pub fn map_colors2_to<I, J, K, P, Q, R, F>(image1: &I, image2: &J, out: &mut K, f: F)
where
    I: GenericImageView<Pixel = P>,
    J: GenericImageView<Pixel = Q>,
    K: GenericImageView<Pixel = R> + GenericImage,
    P: Pixel,
    Q: Pixel,
    R: Pixel,
    F: Fn(u32, u32, P, Q) -> R,
{
    assert_eq!(image1.dimensions(), image2.dimensions());
    assert_eq!(image1.dimensions(), out.dimensions());

    let (width, height) = image1.dimensions();
    for y in 0..height {
        for x in 0..width {
            unsafe {
                let p = image1.unsafe_get_pixel(x, y);
                let q = image2.unsafe_get_pixel(x, y);
                out.unsafe_put_pixel(x, y, f(x, y, p, q));
            }
        }
    }
}
pub fn map_colors2<I, J, P, Q, R, F>(image1: &I, image2: &J, f: F) -> Image<R>
where
    I: GenericImageView<Pixel = P>,
    J: GenericImageView<Pixel = Q>,
    P: Pixel,
    Q: Pixel,
    R: Pixel,
    F: Fn(u32, u32, P, Q) -> R,
{
    let (width, height) = image1.dimensions();
    let mut out: ImageBuffer<R, Vec<R::Subpixel>> = ImageBuffer::new(width, height);
    map_colors2_to(image1, image2, &mut out, f);
    out
}
pub fn map_colors2_in_place<I, J, P, Q, F>(image1: &mut I, image2: &J, f: F)
where
    I: GenericImageView<Pixel = P> + GenericImage,
    J: GenericImageView<Pixel = Q>,
    P: Pixel,
    Q: Pixel,
    F: Fn(u32, u32, P, Q) -> P,
{
    let out = image1 as *mut I;
    unsafe {
        map_colors2_to(image1, image2, &mut *out, f);
    }
}
pub fn map_colors2_parallel_to<I, J, K, P, Q, R, F>(image1: &I, image2: &J, out: &mut K, f: F)
where
    I: GenericImageView<Pixel = P> + Sync,
    J: GenericImageView<Pixel = Q> + Sync,
    K: GenericImageView<Pixel = R> + Sync + GenericImage,
    P: Pixel,
    Q: Pixel,
    R: Pixel + Send + Sync,
    <R as Pixel>::Subpixel: Send + Sync,
    F: Fn(u32, u32, P, Q) -> R + Sync,
{
    assert_eq!(image1.dimensions(), image2.dimensions());
    assert_eq!(image1.dimensions(), out.dimensions());
    use rayon::prelude::*;
    let (width, height) = image1.dimensions();
    let map = |out: &mut K| {
        let out = &*out;
        (0..height).into_par_iter().for_each(|y| {
            for x in 0..width {
                unsafe {
                    let p = image1.unsafe_get_pixel(x, y);
                    let q = image2.unsafe_get_pixel(x, y);
                    let out: *const _ = out as *const _;
                    let out: *mut _ = std::mem::transmute::<_, *mut K>(out);
                    (*out).unsafe_put_pixel(x, y, f(x, y, p, q));
                }
            }
        })
    };
    map(out);
}

pub fn map_colors2_parallel<I, J, P, Q, R, F>(image1: &I, image2: &J, f: F) -> Image<R>
where
    I: GenericImageView<Pixel = P> + Sync,
    J: GenericImageView<Pixel = Q> + Sync,
    P: Pixel,
    Q: Pixel,
    R: Pixel + Send + Sync,
    <R as Pixel>::Subpixel: Send + Sync,
    F: Fn(u32, u32, P, Q) -> R + Sync,
{
    assert_eq!(image1.dimensions(), image2.dimensions());
    let (width, height) = image1.dimensions();
    let mut out: ImageBuffer<R, Vec<R::Subpixel>> = ImageBuffer::new(width, height);
    map_colors2_parallel_to(image1, image2, &mut out, f);
    out
}

pub fn map_colors2_parallel_in_place<I, J, P, Q, R, F>(image1: &mut I, image2: &J, f: F)
where
    I: GenericImageView<Pixel = P> + Sync + GenericImage,
    J: GenericImageView<Pixel = Q> + Sync,
    P: Pixel + Send + Sync,
    Q: Pixel,
    <P as Pixel>::Subpixel: Send + Sync,
    F: Fn(u32, u32, P, Q) -> P + Sync,
{
    assert_eq!(image1.dimensions(), image2.dimensions());
    let out = image1 as *mut I;
    map_colors2_parallel_to(image1, image2, unsafe { &mut *out }, f);
}
