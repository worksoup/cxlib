use std::ops::Deref;

pub type DoesntMatter = Unit;
pub type Unit = RefWrapper<()>;

pub const UNIT: Unit = Unit::__;

impl Unit {
    const __: Self = RefWrapper(());
}
impl Default for Unit {
    fn default() -> Self {
        UNIT
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialOrd, PartialEq)]
#[repr(transparent)]
pub struct RefWrapper<T: ?Sized>(pub T);

impl<T: ?Sized, B: ?Sized + std::borrow::Borrow<T>> AsRef<T> for RefWrapper<B> {
    #[inline]
    fn as_ref(&self) -> &T {
        self.0.borrow()
    }
}
impl<T: ?Sized> Deref for RefWrapper<T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
