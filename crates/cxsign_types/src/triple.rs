#[derive(Debug)]
pub enum Triple<T1, T2, T3> {
    First(T1),
    Second(T2),
    Last(T3),
}
impl<T1, T2, T3> Triple<T1, T2, T3> {
    pub fn into_first(self) -> Option<T1> {
        match self {
            Triple::First(f) => Some(f),
            _ => None,
        }
    }
    pub fn into_second(self) -> Option<T2> {
        match self {
            Triple::Second(s) => Some(s),
            _ => None,
        }
    }
    pub fn into_last(self) -> Option<T3> {
        match self {
            Triple::Last(l) => Some(l),
            _ => None,
        }
    }
    pub fn first(&self) -> Option<&T1> {
        match self {
            Triple::First(f) => Some(f),
            _ => None,
        }
    }
    pub fn second(&self) -> Option<&T2> {
        match self {
            Triple::Second(s) => Some(s),
            _ => None,
        }
    }
    pub fn last(&self) -> Option<&T3> {
        match self {
            Triple::Last(l) => Some(l),
            _ => None,
        }
    }
    pub fn is_first(&self) -> bool {
        matches!(self, Triple::First(_))
    }
    pub fn is_second(&self) -> bool {
        matches!(self, Triple::Second(_))
    }
    pub fn is_last(&self) -> bool {
        matches!(self, Triple::Last(_))
    }
    pub fn map_first<B, FF>(self, ff: FF) -> Triple<B, T2, T3>
    where
        Self: Sized,
        FF: Fn(T1) -> B,
    {
        match self {
            Triple::First(f) => Triple::First(ff(f)),
            Triple::Second(s) => Triple::Second(s),
            Triple::Last(l) => Triple::Last(l),
        }
    }
    pub fn map_second<B, FF>(self, ff: FF) -> Triple<T1, B, T3>
    where
        Self: Sized,
        FF: Fn(T2) -> B,
    {
        match self {
            Triple::First(f) => Triple::First(f),
            Triple::Second(s) => Triple::Second(ff(s)),
            Triple::Last(l) => Triple::Last(l),
        }
    }
    pub fn map_last<B, FF>(self, ff: FF) -> Triple<T1, T2, B>
    where
        Self: Sized,
        FF: Fn(T3) -> B,
    {
        match self {
            Triple::First(f) => Triple::First(f),
            Triple::Second(s) => Triple::Second(s),
            Triple::Last(l) => Triple::Last(ff(l)),
        }
    }
    pub fn set_first(&mut self, value: T1) {
        if let Triple::First(_) = self {
            *self = Triple::First(value);
        }
    }
    pub fn set_second(&mut self, value: T2) {
        if let Triple::Second(_) = self {
            *self = Triple::Second(value);
        }
    }
    pub fn set_last(&mut self, value: T3) {
        if let Triple::Last(_) = self {
            *self = Triple::Last(value);
        }
    }
}
