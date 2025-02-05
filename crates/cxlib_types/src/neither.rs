#[derive(Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum Neither<T1, T2> {
    A(T1),
    B(T2),
}

/////////////////////////////////////////////////////////////////////////////
// Type implementation
/////////////////////////////////////////////////////////////////////////////
impl<T, E> Neither<T, E> {
    /// Converts from `&Neither<T, E>` to `Neither<&T, &E>`.
    #[inline]
    pub const fn as_ref(&self) -> Neither<&T, &E> {
        match *self {
            Self::A(ref x) => Neither::A(x),
            Self::B(ref x) => Neither::B(x),
        }
    }

    /// Converts from `&mut Neither<T, E>` to `Neither<&mut T, &mut E>`.
    #[inline]
    pub const fn as_mut(&mut self) -> Neither<&mut T, &mut E> {
        match *self {
            Self::A(ref mut x) => Neither::A(x),
            Self::B(ref mut x) => Neither::B(x),
        }
    }
}
/////////////////////////////////////////////////////////////////////////////
// Trait implementations
/////////////////////////////////////////////////////////////////////////////

impl<T, E> Clone for Neither<T, E>
where
    T: Clone,
    E: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        match self {
            Self::A(x) => Self::A(x.clone()),
            Self::B(x) => Self::B(x.clone()),
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        match (self, source) {
            (Self::A(to), Self::A(from)) => to.clone_from(from),
            (Self::B(to), Self::B(from)) => to.clone_from(from),
            (to, from) => *to = from.clone(),
        }
    }
}
