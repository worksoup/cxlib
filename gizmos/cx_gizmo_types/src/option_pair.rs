use crate::{OneWithOption, ValidOptionPair};

#[derive(Debug)]
pub struct OptionPair<T1, T2> {
    first: Option<T1>,
    second: Option<T2>,
}
impl<T1, T2> From<(Option<T1>, Option<T2>)> for OptionPair<T1, T2> {
    #[inline]
    fn from(v: (Option<T1>, Option<T2>)) -> Self {
        Self::from_tuple(v)
    }
}
impl<T1, T2> From<(T1, Option<T2>)> for OptionPair<T1, T2> {
    #[inline]
    fn from((first, second): (T1, Option<T2>)) -> Self {
        Self {
            first: Some(first),
            second,
        }
    }
}
impl<T1, T2> From<(Option<T1>, T2)> for OptionPair<T1, T2> {
    #[inline]
    fn from((first, second): (Option<T1>, T2)) -> Self {
        Self {
            first,
            second: Some(second),
        }
    }
}
impl<T1, T2> From<(T1, T2)> for OptionPair<T1, T2> {
    #[inline]
    fn from((first, second): (T1, T2)) -> Self {
        Self {
            first: Some(first),
            second: Some(second),
        }
    }
}
impl<T1, T2> From<ValidOptionPair<T1, T2>> for OptionPair<T1, T2> {
    #[inline]
    fn from(value: ValidOptionPair<T1, T2>) -> Self {
        Self::from_tuple(value.into_tuple())
    }
}
impl<T1, T2> From<OneWithOption<T1, T2>> for OptionPair<T1, T2> {
    #[inline]
    fn from(value: OneWithOption<T1, T2>) -> Self {
        value.into_tuple().into()
    }
}
impl<T1, T2> OptionPair<T1, T2> {
    #[inline]
    pub fn new_none() -> Self {
        Self {
            first: None,
            second: None,
        }
    }
    #[inline]
    pub fn new_first(f: T1) -> Self {
        Self {
            first: Some(f),
            second: None,
        }
    }
    #[inline]
    pub fn new_second(s: T2) -> Self {
        Self {
            first: None,
            second: Some(s),
        }
    }
    #[inline]
    pub fn new_both(f: T1, s: T2) -> Self {
        Self {
            first: Some(f),
            second: Some(s),
        }
    }
    #[inline]
    pub fn as_ref(&self) -> OptionPair<&T1, &T2> {
        OptionPair {
            first: self.first.as_ref(),
            second: self.second.as_ref(),
        }
    }
    #[inline]
    pub fn as_mut(&mut self) -> OptionPair<&mut T1, &mut T2> {
        OptionPair {
            first: self.first.as_mut(),
            second: self.second.as_mut(),
        }
    }
    #[inline]
    pub fn from_tuple((first, second): (Option<T1>, Option<T2>)) -> Self {
        Self { first, second }
    }
    #[inline]
    pub fn take_first(&mut self) -> Option<T1> {
        self.first.take()
    }
    #[inline]
    pub fn take_second(&mut self) -> Option<T2> {
        self.second.take()
    }
    #[inline]
    pub fn into_first(self) -> Option<T1> {
        self.first
    }
    #[inline]
    pub fn into_second(self) -> Option<T2> {
        self.second
    }
    #[inline]
    pub fn into_both(self) -> Option<(T1, T2)> {
        if let Self {
            first: Some(first),
            second: Some(second),
        } = self
        {
            Some((first, second))
        } else {
            None
        }
    }
    #[inline]
    pub fn into_tuple(self) -> (Option<T1>, Option<T2>) {
        (self.first, self.second)
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
    pub fn is_none(&self) -> bool {
        self.first.is_some() && self.second.is_some()
    }
    #[inline]
    pub fn is_first(&self) -> bool {
        self.first.is_some() && self.second.is_none()
    }
    #[inline]
    pub fn is_second(&self) -> bool {
        self.second.is_some() && self.first.is_none()
    }
    #[inline]
    pub fn is_both(&self) -> bool {
        self.first.is_some() && self.second.is_some()
    }
    #[inline]
    pub fn has_first(&self) -> bool {
        self.first.is_some()
    }
    #[inline]
    pub fn has_second(&self) -> bool {
        self.second.is_some()
    }
    #[inline]
    pub fn map<B1, B2, FF, SF>(self, mut ff: FF, mut sf: SF) -> OptionPair<B1, B2>
    where
        Self: Sized,
        FF: FnMut(T1) -> B1,
        SF: FnMut(T2) -> B2,
    {
        let first = self.first.map(&mut ff);
        let second = self.second.map(&mut sf);
        OptionPair { first, second }
    }
    #[inline]
    pub fn map_first<B, FF>(self, ff: FF) -> OptionPair<B, T2>
    where
        Self: Sized,
        FF: FnMut(T1) -> B,
    {
        self.map(ff, |s| s)
    }
    #[inline]
    pub fn map_second<B, F>(self, f: F) -> OptionPair<T1, B>
    where
        Self: Sized,
        F: FnMut(T2) -> B,
    {
        self.map(|f| f, f)
    }
    #[inline]
    pub fn push_first(&mut self, value: T1) -> Option<T1> {
        self.first.replace(value)
    }
    #[inline]
    pub fn push_second(&mut self, value: T2) -> Option<T2> {
        self.second.replace(value)
    }
    #[inline]
    pub fn set_first(&mut self, value: T1) {
        self.first = Some(value);
    }
    #[inline]
    pub fn set_second(&mut self, value: T2) {
        self.second = Some(value);
    }
}

impl<T> OptionPair<T, T> {
    #[inline]
    pub fn push(&mut self, value: T) -> Option<T> {
        if !self.has_first() {
            self.first = Some(value);
            None
        } else if !self.has_second() {
            self.second = Some(value);
            None
        } else {
            Some(value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_option_pair() {
        let mut a = OptionPair::new_both(1, 2);
        let b = a.take_first();
        println!("a: {a:?}, b: {b:?}");
    }
}
