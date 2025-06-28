#[derive(Debug)]
pub enum TripleVariant<T1, T2, T3> {
    First(T1),
    Second(T2),
    Last(T3),
}
impl<T1, T2, T3> TripleVariant<T1, T2, T3> {
    #[inline]
    pub fn into_first(self) -> Option<T1> {
        match self {
            TripleVariant::First(f) => Some(f),
            _ => None,
        }
    }
    #[inline]
    pub fn into_second(self) -> Option<T2> {
        match self {
            TripleVariant::Second(s) => Some(s),
            _ => None,
        }
    }
    #[inline]
    pub fn into_last(self) -> Option<T3> {
        match self {
            TripleVariant::Last(l) => Some(l),
            _ => None,
        }
    }
    #[inline]
    pub fn first(&self) -> Option<&T1> {
        match self {
            TripleVariant::First(f) => Some(f),
            _ => None,
        }
    }
    #[inline]
    pub fn second(&self) -> Option<&T2> {
        match self {
            TripleVariant::Second(s) => Some(s),
            _ => None,
        }
    }
    #[inline]
    pub fn last(&self) -> Option<&T3> {
        match self {
            TripleVariant::Last(l) => Some(l),
            _ => None,
        }
    }
    #[inline]
    pub fn is_first(&self) -> bool {
        matches!(self, TripleVariant::First(_))
    }
    #[inline]
    pub fn is_second(&self) -> bool {
        matches!(self, TripleVariant::Second(_))
    }
    #[inline]
    pub fn is_last(&self) -> bool {
        matches!(self, TripleVariant::Last(_))
    }
    #[inline]
    pub fn map_first<B, FF>(self, ff: FF) -> TripleVariant<B, T2, T3>
    where
        Self: Sized,
        FF: Fn(T1) -> B,
    {
        match self {
            TripleVariant::First(f) => TripleVariant::First(ff(f)),
            TripleVariant::Second(s) => TripleVariant::Second(s),
            TripleVariant::Last(l) => TripleVariant::Last(l),
        }
    }
    #[inline]
    pub fn map_second<B, FF>(self, ff: FF) -> TripleVariant<T1, B, T3>
    where
        Self: Sized,
        FF: Fn(T2) -> B,
    {
        match self {
            TripleVariant::First(f) => TripleVariant::First(f),
            TripleVariant::Second(s) => TripleVariant::Second(ff(s)),
            TripleVariant::Last(l) => TripleVariant::Last(l),
        }
    }
    #[inline]
    pub fn map_last<B, FF>(self, ff: FF) -> TripleVariant<T1, T2, B>
    where
        Self: Sized,
        FF: Fn(T3) -> B,
    {
        match self {
            TripleVariant::First(f) => TripleVariant::First(f),
            TripleVariant::Second(s) => TripleVariant::Second(s),
            TripleVariant::Last(l) => TripleVariant::Last(ff(l)),
        }
    }
    #[inline]
    pub fn set_first(&mut self, value: T1) {
        if let TripleVariant::First(_) = self {
            *self = TripleVariant::First(value);
        }
    }
    #[inline]
    pub fn set_second(&mut self, value: T2) {
        if let TripleVariant::Second(_) = self {
            *self = TripleVariant::Second(value);
        }
    }
    #[inline]
    pub fn set_last(&mut self, value: T3) {
        if let TripleVariant::Last(_) = self {
            *self = TripleVariant::Last(value);
        }
    }
}
