use std::borrow::Borrow;

pub trait TryFromWithContext<T>: Sized {
    type Err;
    type Context<'cxt>: ?Sized;
    fn try_from<'cxt, Cxt: Borrow<Self::Context<'cxt>>>(s: T, cxt: Cxt) -> Result<Self, Self::Err>;
}
pub trait FromStrWithContext<'a>: TryFromWithContext<&'a str> {
    fn from_str<'cxt, Cxt: Borrow<Self::Context<'cxt>>>(
        s: &'a str,
        cxt: Cxt,
    ) -> Result<Self, Self::Err> {
        TryFromWithContext::try_from(s, cxt)
    }
}
impl<'a, T: TryFromWithContext<&'a str>> FromStrWithContext<'a> for T {}
