mod rotate_images;
mod slide_images;

pub use rotate_images::*;
pub use slide_images::*;

pub trait SolverProviderTrait {
    type I: 'static;
    type O: 'static;
    fn solver(input: Self::I) -> Result<Self::O, crate::CaptchaError>;
}
