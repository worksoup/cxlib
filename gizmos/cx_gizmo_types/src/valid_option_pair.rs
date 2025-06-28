use crate::{OneWithOption, OptionPair};

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("attempt to construct Self from none value.")]
    FoundNoneValue,
    #[error("cannot take this while self is to be none value.")]
    CannotTakeThisValue,
}

#[derive(Debug)]
pub enum ValidOptionPair<T1, T2> {
    First(T1),
    Second(T2),
    Both(T1, T2),
}
impl<T1, T2> TryFrom<OptionPair<T1, T2>> for ValidOptionPair<T1, T2> {
    type Error = Error;

    #[inline]
    fn try_from(value: OptionPair<T1, T2>) -> Result<Self, Self::Error> {
        Self::from_tuple(value.into_tuple())
    }
}
impl<T1, T2> TryFrom<(Option<T1>, Option<T2>)> for ValidOptionPair<T1, T2> {
    type Error = Error;

    #[inline]
    fn try_from(value: (Option<T1>, Option<T2>)) -> Result<Self, Self::Error> {
        Self::from_tuple(value)
    }
}
impl<T1, T2> From<(T1, Option<T2>)> for ValidOptionPair<T1, T2> {
    #[inline]
    fn from((first, second): (T1, Option<T2>)) -> Self {
        if let Some(second) = second {
            Self::Both(first, second)
        } else {
            Self::First(first)
        }
    }
}
impl<T1, T2> From<(Option<T1>, T2)> for ValidOptionPair<T1, T2> {
    #[inline]
    fn from((first, second): (Option<T1>, T2)) -> Self {
        if let Some(first) = first {
            Self::Both(first, second)
        } else {
            Self::Second(second)
        }
    }
}
impl<T1, T2> From<(T1, T2)> for ValidOptionPair<T1, T2> {
    #[inline]
    fn from((first, second): (T1, T2)) -> Self {
        Self::Both(first, second)
    }
}
impl<T1, T2> From<OneWithOption<T1, T2>> for ValidOptionPair<T1, T2> {
    #[inline]
    fn from(value: OneWithOption<T1, T2>) -> Self {
        value.into_tuple().into()
    }
}
impl<T1, T2> ValidOptionPair<T1, T2> {
    #[inline]
    pub fn new_first(f: T1) -> Self {
        Self::First(f)
    }
    #[inline]
    pub fn new_second(s: T2) -> Self {
        Self::Second(s)
    }
    #[inline]
    pub fn new_both(f: T1, s: T2) -> Self {
        Self::Both(f, s)
    }
    #[inline]
    pub fn from_tuple(t: (Option<T1>, Option<T2>)) -> Result<Self, Error> {
        match t {
            (Some(first), Some(second)) => Ok(Self::Both(first, second)),
            (Some(first), None) => Ok(Self::First(first)),
            (None, Some(second)) => Ok(Self::Second(second)),
            (None, None) => Err(Error::FoundNoneValue),
        }
    }
    #[inline]
    pub fn as_ref(&self) -> ValidOptionPair<&T1, &T2> {
        match self {
            ValidOptionPair::Both(first, second) => ValidOptionPair::Both(first, second),
            ValidOptionPair::First(first) => ValidOptionPair::First(first),
            ValidOptionPair::Second(second) => ValidOptionPair::Second(second),
        }
    }
    #[inline]
    pub fn as_mut(&mut self) -> ValidOptionPair<&mut T1, &mut T2> {
        match self {
            ValidOptionPair::Both(first, second) => ValidOptionPair::Both(first, second),
            ValidOptionPair::First(first) => ValidOptionPair::First(first),
            ValidOptionPair::Second(second) => ValidOptionPair::Second(second),
        }
    }
    #[inline]
    pub fn take_first(&mut self) -> Result<Option<T1>, Error> {
        match self {
            ValidOptionPair::First(_) => Err(Error::CannotTakeThisValue),
            ValidOptionPair::Second(_) => Ok(None),
            ValidOptionPair::Both(first, second) => unsafe {
                let first = { std::ptr::read(first) };
                let second = { std::ptr::read(second) };
                std::ptr::write(self, ValidOptionPair::Second(second));
                Ok(Some(first))
            },
        }
    }
    #[inline]
    pub fn take_second(&mut self) -> Result<Option<T2>, Error> {
        match self {
            ValidOptionPair::First(_) => Ok(None),
            ValidOptionPair::Second(_) => Err(Error::CannotTakeThisValue),
            ValidOptionPair::Both(first, second) => unsafe {
                let first = { std::ptr::read(first) };
                let second = { std::ptr::read(second) };
                std::ptr::write(self, ValidOptionPair::First(first));
                Ok(Some(second))
            },
        }
    }
    #[inline]
    pub fn into_first(self) -> Option<T1> {
        match self {
            ValidOptionPair::Both(first, _) | ValidOptionPair::First(first) => Some(first),
            ValidOptionPair::Second(_) => None,
        }
    }
    #[inline]
    pub fn into_second(self) -> Option<T2> {
        match self {
            ValidOptionPair::Both(_, second) | ValidOptionPair::Second(second) => Some(second),
            ValidOptionPair::First(_) => None,
        }
    }
    #[inline]
    pub fn into_both(self) -> Option<(T1, T2)> {
        match self {
            ValidOptionPair::Both(first, second) => Some((first, second)),
            ValidOptionPair::First(_) | ValidOptionPair::Second(_) => None,
        }
    }
    #[inline]
    pub fn into_tuple(self) -> (Option<T1>, Option<T2>) {
        match self {
            ValidOptionPair::Both(first, second) => (Some(first), Some(second)),
            ValidOptionPair::First(first) => (Some(first), None),
            ValidOptionPair::Second(second) => (None, Some(second)),
        }
    }
    #[inline]
    pub fn first(&self) -> Option<&T1> {
        self.as_ref().into_first()
    }
    #[inline]
    pub fn second(&self) -> Option<&T2> {
        self.as_ref().into_second()
    }
    #[inline]
    pub fn both(&self) -> Option<(&T1, &T2)> {
        self.as_ref().into_both()
    }
    #[inline]
    pub fn tuple(&self) -> (Option<&T1>, Option<&T2>) {
        self.as_ref().into_tuple()
    }
    #[inline]
    pub fn first_mut(&mut self) -> Option<&mut T1> {
        self.as_mut().into_first()
    }
    #[inline]
    pub fn second_mut(&mut self) -> Option<&mut T2> {
        self.as_mut().into_second()
    }
    #[inline]
    pub fn both_mut(&mut self) -> Option<(&mut T1, &mut T2)> {
        self.as_mut().into_both()
    }
    #[inline]
    pub fn tuple_mut(&mut self) -> (Option<&mut T1>, Option<&mut T2>) {
        self.as_mut().into_tuple()
    }
    #[inline]
    pub fn is_first(&self) -> bool {
        match self {
            ValidOptionPair::First(_) => true,
            ValidOptionPair::Second(_) | ValidOptionPair::Both(_, _) => false,
        }
    }
    #[inline]
    pub fn is_second(&self) -> bool {
        match self {
            ValidOptionPair::Second(_) => true,
            ValidOptionPair::First(_) | ValidOptionPair::Both(_, _) => false,
        }
    }
    #[inline]
    pub fn has_first(&self) -> bool {
        match self {
            ValidOptionPair::First(_) | ValidOptionPair::Both(_, _) => true,
            ValidOptionPair::Second(_) => false,
        }
    }
    #[inline]
    pub fn is_both(&self) -> bool {
        self.has_first() && self.has_second()
    }
    #[inline]
    pub fn has_second(&self) -> bool {
        match self {
            ValidOptionPair::Second(_) | ValidOptionPair::Both(_, _) => true,
            ValidOptionPair::First(_) => false,
        }
    }
    #[inline]
    pub fn map<B1, B2, FF, SF>(self, mut ff: FF, mut sf: SF) -> ValidOptionPair<B1, B2>
    where
        Self: Sized,
        FF: FnMut(T1) -> B1,
        SF: FnMut(T2) -> B2,
    {
        match self {
            ValidOptionPair::First(first) => ValidOptionPair::First(ff(first)),
            ValidOptionPair::Second(second) => ValidOptionPair::Second(sf(second)),
            ValidOptionPair::Both(first, second) => ValidOptionPair::Both(ff(first), sf(second)),
        }
    }
    #[inline]
    pub fn map_first<B, F>(self, f: F) -> ValidOptionPair<B, T2>
    where
        Self: Sized,
        F: FnMut(T1) -> B,
    {
        self.map(f, |s| s)
    }
    #[inline]
    pub fn map_second<B, F>(self, f: F) -> ValidOptionPair<T1, B>
    where
        Self: Sized,
        F: FnMut(T2) -> B,
    {
        self.map(|f| f, f)
    }
    #[inline]
    pub fn push_first(&mut self, value: T1) -> Option<T1> {
        match self {
            ValidOptionPair::Second(second) => unsafe {
                let both = ValidOptionPair::Both(value, std::ptr::read(second));
                std::ptr::write(self, both);
                None
            },
            ValidOptionPair::First(_) | ValidOptionPair::Both(_, _) => Some(value),
        }
    }
    #[inline]
    pub fn push_second(&mut self, value: T2) -> Option<T2> {
        match self {
            ValidOptionPair::First(first) => unsafe {
                let both = ValidOptionPair::Both(std::ptr::read(first), value);
                std::ptr::write(self, both);
                None
            },
            ValidOptionPair::Second(_) | ValidOptionPair::Both(_, _) => Some(value),
        }
    }
    #[inline]
    pub fn set_first(&mut self, value: T1) {
        match self {
            ValidOptionPair::Both(first, _) | ValidOptionPair::First(first) => {
                *first = value;
            }
            ValidOptionPair::Second(second) => {
                let mut both = ValidOptionPair::Both(value, unsafe { std::ptr::read(second) });
                std::mem::swap(self, &mut both);
            }
        }
    }
    #[inline]
    pub fn set_second(&mut self, value: T2) {
        match self {
            ValidOptionPair::First(first) => {
                let mut both = ValidOptionPair::Both(unsafe { std::ptr::read(first) }, value);
                std::mem::swap(self, &mut both);
            }
            ValidOptionPair::Second(second) | ValidOptionPair::Both(_, second) => {
                *second = value;
            }
        }
    }
}

impl<T> ValidOptionPair<T, T> {
    #[inline]
    pub fn push(&mut self, value: T) -> Option<T> {
        match self {
            ValidOptionPair::First(first) => {
                let mut both = ValidOptionPair::Both(unsafe { std::ptr::read(first) }, value);
                std::mem::swap(self, &mut both);
                None
            }
            ValidOptionPair::Second(second) => {
                let mut both = ValidOptionPair::Both(value, unsafe { std::ptr::read(second) });
                std::mem::swap(self, &mut both);
                None
            }
            ValidOptionPair::Both(_, _) => Some(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_option_pair() {
        let mut a = ValidOptionPair::new_both(1, 2);
        let b = a.take_first();
        println!("a: {a:?}, b: {b:?}");
    }
}
