#[repr(transparent)]
pub struct MySyncUnsafeCell<T: ?Sized> {
    value: std::cell::UnsafeCell<T>,
}
unsafe impl<T: ?Sized + Sync> Sync for MySyncUnsafeCell<T> {}
impl<T> MySyncUnsafeCell<T> {
    #[inline]
    pub const fn new(value: T) -> Self {
        Self {
            value: std::cell::UnsafeCell::new(value),
        }
    }
    #[inline]
    pub fn into_inner(self) -> T {
        self.value.into_inner()
    }
    #[inline]
    pub const fn from_mut(value: &mut T) -> &mut MySyncUnsafeCell<T> {
        // SAFETY: `MySyncUnsafeCell<T>` has the same memory layout as `T` due to #[repr(transparent)].
        unsafe { &mut *(value as *mut T as *mut MySyncUnsafeCell<T>) }
    }
}
impl<T: ?Sized> MySyncUnsafeCell<T> {
    #[inline]
    pub const fn get(&self) -> *mut T {
        self.value.get()
    }
}
impl<T> From<T> for MySyncUnsafeCell<T> {
    #[inline]
    fn from(t: T) -> MySyncUnsafeCell<T> {
        MySyncUnsafeCell::new(t)
    }
}
