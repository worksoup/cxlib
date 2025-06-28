use std::ops::Deref;

pub enum OneWithOption<T1, T2> {
    One(T1),
    WithOther(T1, T2),
}
impl<T1, T2> Deref for OneWithOption<T1, T2> {
    type Target = T1;

    fn deref(&self) -> &Self::Target {
        self.one()
    }
}
impl<T1, T2> From<T1> for OneWithOption<T1, T2> {
    #[inline]
    fn from(value: T1) -> Self {
        Self::One(value)
    }
}
impl<T1, T2> From<(T1, Option<T2>)> for OneWithOption<T1, T2> {
    #[inline]
    fn from(value: (T1, Option<T2>)) -> Self {
        Self::from_tuple(value)
    }
}
impl<T1, T2> From<(T1, T2)> for OneWithOption<T1, T2> {
    #[inline]
    fn from((one, other): (T1, T2)) -> Self {
        Self::WithOther(one, other)
    }
}
impl<T1, T2> OneWithOption<T1, T2> {
    #[inline]
    pub fn new(t1: T1, t2: Option<T2>) -> Self {
        if let Some(t2) = t2 {
            Self::WithOther(t1, t2)
        } else {
            Self::One(t1)
        }
    }
    #[inline]
    pub fn new_one(t1: T1) -> Self {
        Self::One(t1)
    }
    #[inline]
    pub fn from_tuple(t: (T1, Option<T2>)) -> Self {
        match t {
            (t1, Some(t2)) => Self::WithOther(t1, t2),
            (t1, None) => Self::One(t1),
        }
    }
    #[inline]
    pub fn as_ref(&self) -> OneWithOption<&T1, &T2> {
        match self {
            OneWithOption::One(one) => OneWithOption::One(one),
            OneWithOption::WithOther(one, other) => OneWithOption::WithOther(one, other),
        }
    }
    #[inline]
    pub fn as_mut(&mut self) -> OneWithOption<&mut T1, &mut T2> {
        match self {
            OneWithOption::One(one) => OneWithOption::One(one),
            OneWithOption::WithOther(one, other) => OneWithOption::WithOther(one, other),
        }
    }
    #[inline]
    pub fn take_other(&mut self) -> Option<T2> {
        match self {
            OneWithOption::One(_) => None,
            OneWithOption::WithOther(one, other) => unsafe {
                let one = std::ptr::read(one);
                let other = std::ptr::read(other);
                std::ptr::write(self, OneWithOption::One(one));
                Some(other)
            },
        }
    }
    #[inline]
    pub fn into_one(self) -> T1 {
        match self {
            OneWithOption::One(one) | OneWithOption::WithOther(one, _) => one,
        }
    }
    #[inline]
    pub fn into_other(self) -> Option<T2> {
        match self {
            OneWithOption::One(_) => None,
            OneWithOption::WithOther(_, other) => Some(other),
        }
    }
    #[inline]
    pub fn into_tuple(self) -> (T1, Option<T2>) {
        match self {
            OneWithOption::One(one) => (one, None),
            OneWithOption::WithOther(one, other) => (one, Some(other)),
        }
    }
    #[inline]
    pub fn one(&self) -> &T1 {
        self.as_ref().into_one()
    }
    #[inline]
    pub fn other(&self) -> Option<&T2> {
        self.as_ref().into_other()
    }
    #[inline]
    pub fn tuple(&self) -> (&T1, Option<&T2>) {
        self.as_ref().into_tuple()
    }
    #[inline]
    pub fn one_mut(&mut self) -> &mut T1 {
        self.as_mut().into_one()
    }
    #[inline]
    pub fn other_mut(&mut self) -> Option<&mut T2> {
        self.as_mut().into_other()
    }
    #[inline]
    pub fn tuple_mut(&mut self) -> (&mut T1, Option<&mut T2>) {
        self.as_mut().into_tuple()
    }
    #[inline]
    pub fn has_other(&self) -> bool {
        match self {
            OneWithOption::One(_) => false,
            OneWithOption::WithOther(_, _) => true,
        }
    }
    #[inline]
    pub fn map<B1, B2, FF, SF>(self, mut ff: FF, mut sf: SF) -> OneWithOption<B1, B2>
    where
        Self: Sized,
        FF: FnMut(T1) -> B1,
        SF: FnMut(T2) -> B2,
    {
        match self {
            Self::One(first) => OneWithOption::One(ff(first)),
            Self::WithOther(first, second) => OneWithOption::WithOther(ff(first), sf(second)),
        }
    }
    #[inline]
    pub fn map_one<B, F>(self, f: F) -> OneWithOption<B, T2>
    where
        Self: Sized,
        F: FnMut(T1) -> B,
    {
        self.map(f, |s| s)
    }
    #[inline]
    pub fn map_other<B, F>(self, f: F) -> OneWithOption<T1, B>
    where
        Self: Sized,
        F: FnMut(T2) -> B,
    {
        self.map(|f| f, f)
    }
    #[inline]
    pub fn push(&mut self, other: T2) -> Option<T2> {
        match self {
            OneWithOption::One(one) => unsafe {
                let one = std::ptr::read(one);
                std::ptr::write(self, OneWithOption::WithOther(one, other));
                None
            },
            OneWithOption::WithOther(_, _) => Some(other),
        }
    }
    #[inline]
    pub fn set_one(&mut self, value: T1) {
        match self {
            OneWithOption::One(one) | OneWithOption::WithOther(one, _) => {
                *one = value;
            }
        }
    }
    #[inline]
    pub fn set_other(&mut self, value: T2) {
        match self {
            OneWithOption::One(_) => {}
            OneWithOption::WithOther(_, other) => {
                *other = value;
            }
        }
    }
}
