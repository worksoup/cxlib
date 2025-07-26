use crate::{OneWithOption, OptionPair};

/// AI生成：选项对操作过程中可能出现的错误类型
#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    /// AI生成：尝试从空值构造有效选项对时发生
    #[error("attempt to construct Self from none value.")]
    FoundNoneValue,
    /// AI生成：尝试从无效状态取出值时发生
    #[error("cannot take this while self is to be none value.")]
    CannotTakeThisValue,
}

/// AI生成：`ValidOptionPair` 枚举表示必须至少包含一个值的有效选项对
///
/// 与`OptionPair`不同，此类型保证不会出现两个值都为空的情况
/// 提供安全访问方法和状态转换接口
/// 典型应用场景：需要保证至少存在一个值的双选项数据结构
#[derive(Debug)]
pub enum ValidOptionPair<T1, T2> {
    /// AI生成：仅包含第一个值的情况
    First(T1),
    /// AI生成：仅包含第二个值的情况
    Second(T2),
    /// AI生成：同时包含两个值的情况
    Both(T1, T2),
}
impl<T1, T2> TryFrom<OptionPair<T1, T2>> for ValidOptionPair<T1, T2> {
    type Error = Error;

    /// AI生成：尝试从`OptionPair`转换，仅当至少有一个值时成功
    #[inline]
    fn try_from(value: OptionPair<T1, T2>) -> Result<Self, Self::Error> {
        Self::from_tuple(value.into_tuple())
    }
}

impl<T1, T2> TryFrom<(Option<T1>, Option<T2>)> for ValidOptionPair<T1, T2> {
    type Error = Error;

    /// AI生成：尝试从选项元组转换，仅当至少有一个值时成功
    #[inline]
    fn try_from(value: (Option<T1>, Option<T2>)) -> Result<Self, Self::Error> {
        Self::from_tuple(value)
    }
}

impl<T1, T2> From<(T1, Option<T2>)> for ValidOptionPair<T1, T2> {
    /// AI生成：从(值, Option)元组转换，自动处理有效状态
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
    /// AI生成：从(Option, 值)元组转换，自动处理有效状态
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
    /// AI生成：从普通值元组转换，自动创建双值状态
    #[inline]
    fn from((first, second): (T1, T2)) -> Self {
        Self::Both(first, second)
    }
}

impl<T1, T2> From<OneWithOption<T1, T2>> for ValidOptionPair<T1, T2> {
    /// AI生成：从`OneWithOption`类型转换
    #[inline]
    fn from(value: OneWithOption<T1, T2>) -> Self {
        value.into_tuple().into()
    }
}

impl<T1, T2> ValidOptionPair<T1, T2> {
    /// AI生成：创建仅包含第一个值的实例
    #[inline]
    pub fn new_first(f: T1) -> Self {
        Self::First(f)
    }

    /// AI生成：创建仅包含第二个值的实例
    #[inline]
    pub fn new_second(s: T2) -> Self {
        Self::Second(s)
    }

    /// AI生成：创建同时包含两个值的实例
    #[inline]
    pub fn new_both(f: T1, s: T2) -> Self {
        Self::Both(f, s)
    }

    /// AI生成：从选项元组构造实例，失败返回错误
    ///
    /// # 错误
    /// 当元组中两个值均为空时返回`Error::FoundNoneValue`
    #[inline]
    pub fn from_tuple(t: (Option<T1>, Option<T2>)) -> Result<Self, Error> {
        match t {
            (Some(first), Some(second)) => Ok(Self::Both(first, second)),
            (Some(first), None) => Ok(Self::First(first)),
            (None, Some(second)) => Ok(Self::Second(second)),
            (None, None) => Err(Error::FoundNoneValue),
        }
    }

    /// AI生成：获取内部值的不可变引用
    #[inline]
    pub fn as_ref(&self) -> ValidOptionPair<&T1, &T2> {
        match self {
            ValidOptionPair::Both(first, second) => ValidOptionPair::Both(first, second),
            ValidOptionPair::First(first) => ValidOptionPair::First(first),
            ValidOptionPair::Second(second) => ValidOptionPair::Second(second),
        }
    }

    /// AI生成：获取内部值的可变引用
    #[inline]
    pub fn as_mut(&mut self) -> ValidOptionPair<&mut T1, &mut T2> {
        match self {
            ValidOptionPair::Both(first, second) => ValidOptionPair::Both(first, second),
            ValidOptionPair::First(first) => ValidOptionPair::First(first),
            ValidOptionPair::Second(second) => ValidOptionPair::Second(second),
        }
    }

    /// AI生成：取出第一个值并更新自身状态
    ///
    /// # 行为
    /// - 在`Both`状态：取出第一个值，自身转为`Second`状态
    /// - 在`First`状态：返回错误（无法取出唯一值）
    /// - 在`Second`状态：返回`Ok(None)`
    ///
    /// # 错误
    /// 当处于`First`状态时返回`Error::CannotTakeThisValue`
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

    /// AI生成：取出第二个值并更新自身状态
    ///
    /// # 行为
    /// - 在`Both`状态：取出第二个值，自身转为`First`状态
    /// - 在`Second`状态：返回错误（无法取出唯一值）
    /// - 在`First`状态：返回`Ok(None)`
    ///
    /// # 错误
    /// 当处于`Second`状态时返回`Error::CannotTakeThisValue`
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

    /// AI生成：消费实例并返回第一个值（如存在）
    #[inline]
    pub fn into_first(self) -> Option<T1> {
        match self {
            ValidOptionPair::Both(first, _) | ValidOptionPair::First(first) => Some(first),
            ValidOptionPair::Second(_) => None,
        }
    }

    /// AI生成：消费实例并返回第二个值（如存在）
    #[inline]
    pub fn into_second(self) -> Option<T2> {
        match self {
            ValidOptionPair::Both(_, second) | ValidOptionPair::Second(second) => Some(second),
            ValidOptionPair::First(_) => None,
        }
    }

    /// AI生成：消费实例并返回两个值（仅在双值状态）
    #[inline]
    pub fn into_both(self) -> Option<(T1, T2)> {
        match self {
            ValidOptionPair::Both(first, second) => Some((first, second)),
            ValidOptionPair::First(_) | ValidOptionPair::Second(_) => None,
        }
    }

    /// AI生成：消费实例并返回元组形式
    #[inline]
    pub fn into_tuple(self) -> (Option<T1>, Option<T2>) {
        match self {
            ValidOptionPair::Both(first, second) => (Some(first), Some(second)),
            ValidOptionPair::First(first) => (Some(first), None),
            ValidOptionPair::Second(second) => (None, Some(second)),
        }
    }

    /// AI生成：获取第一个值的不可变引用（如存在）
    #[inline]
    pub fn first(&self) -> Option<&T1> {
        self.as_ref().into_first()
    }

    /// AI生成：获取第二个值的不可变引用（如存在）
    #[inline]
    pub fn second(&self) -> Option<&T2> {
        self.as_ref().into_second()
    }

    /// AI生成：获取两个值的不可变引用（仅在双值状态）
    #[inline]
    pub fn both(&self) -> Option<(&T1, &T2)> {
        self.as_ref().into_both()
    }

    /// AI生成：获取元组形式的不可变引用
    #[inline]
    pub fn tuple(&self) -> (Option<&T1>, Option<&T2>) {
        self.as_ref().into_tuple()
    }

    /// AI生成：获取第一个值的可变引用（如存在）
    #[inline]
    pub fn first_mut(&mut self) -> Option<&mut T1> {
        self.as_mut().into_first()
    }

    /// AI生成：获取第二个值的可变引用（如存在）
    #[inline]
    pub fn second_mut(&mut self) -> Option<&mut T2> {
        self.as_mut().into_second()
    }

    /// AI生成：获取两个值的可变引用（仅在双值状态）
    #[inline]
    pub fn both_mut(&mut self) -> Option<(&mut T1, &mut T2)> {
        self.as_mut().into_both()
    }

    /// AI生成：获取元组形式的可变引用
    #[inline]
    pub fn tuple_mut(&mut self) -> (Option<&mut T1>, Option<&mut T2>) {
        self.as_mut().into_tuple()
    }

    /// AI生成：检查是否仅包含第一个值
    #[inline]
    pub fn is_first(&self) -> bool {
        matches!(self, ValidOptionPair::First(_))
    }

    /// AI生成：检查是否仅包含第二个值
    #[inline]
    pub fn is_second(&self) -> bool {
        matches!(self, ValidOptionPair::Second(_))
    }

    /// AI生成：检查是否包含第一个值
    #[inline]
    pub fn has_first(&self) -> bool {
        !self.is_second()
    }

    /// AI生成：检查是否包含第二个值
    #[inline]
    pub fn has_second(&self) -> bool {
        !self.is_first()
    }

    /// AI生成：检查是否同时包含两个值
    #[inline]
    pub fn is_both(&self) -> bool {
        matches!(self, ValidOptionPair::Both(_, _))
    }

    /// AI生成：映射两个值到新类型
    ///
    /// 根据当前状态应用相应的转换函数
    ///
    /// # 示例
    /// ```
    /// let pair = ValidOptionPair::First(42);
    /// let result = pair.map(|n| n * 2, |s| s.len());
    /// assert!(matches!(result, ValidOptionPair::First(84)));
    /// ```
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

    /// AI生成：仅映射第一个值到新类型
    #[inline]
    pub fn map_first<B, F>(self, mut f: F) -> ValidOptionPair<B, T2>
    where
        Self: Sized,
        F: FnMut(T1) -> B,
    {
        match self {
            ValidOptionPair::First(first) => ValidOptionPair::First(f(first)),
            ValidOptionPair::Second(second) => ValidOptionPair::Second(second),
            ValidOptionPair::Both(first, second) => ValidOptionPair::Both(f(first), second),
        }
    }

    /// AI生成：仅映射第二个值到新类型
    #[inline]
    pub fn map_second<B, F>(self, mut f: F) -> ValidOptionPair<T1, B>
    where
        Self: Sized,
        F: FnMut(T2) -> B,
    {
        match self {
            ValidOptionPair::First(first) => ValidOptionPair::First(first),
            ValidOptionPair::Second(second) => ValidOptionPair::Second(f(second)),
            ValidOptionPair::Both(first, second) => ValidOptionPair::Both(first, f(second)),
        }
    }

    /// AI生成：尝试添加第一个值并返回可能溢出的值
    ///
    /// # 行为
    /// - 在`Second`状态：转换为`Both`状态，返回`None`
    /// - 其他状态：返回传入的值(`Some`)
    #[inline]
    pub fn push_first(&mut self, value: T1) -> Option<T1> {
        match self {
            ValidOptionPair::Second(second) => unsafe {
                let both = ValidOptionPair::Both(value, std::ptr::read(second));
                std::ptr::write(self, both);
                None
            },
            _ => Some(value),
        }
    }

    /// AI生成：尝试添加第二个值并返回可能溢出的值
    ///
    /// # 行为
    /// - 在`First`状态：转换为`Both`状态，返回`None`
    /// - 其他状态：返回传入的值(`Some`)
    #[inline]
    pub fn push_second(&mut self, value: T2) -> Option<T2> {
        match self {
            ValidOptionPair::First(first) => unsafe {
                let both = ValidOptionPair::Both(std::ptr::read(first), value);
                std::ptr::write(self, both);
                None
            },
            _ => Some(value),
        }
    }

    /// AI生成：设置或添加第一个值
    ///
    /// # 行为
    /// - 在`Both`/`First`状态：直接覆盖第一个值
    /// - 在`Second`状态：转换为`Both`状态
    #[inline]
    pub fn set_first(&mut self, value: T1) {
        match self {
            ValidOptionPair::Both(first, _) | ValidOptionPair::First(first) => {
                *first = value;
            }
            ValidOptionPair::Second(second) => unsafe {
                let both = ValidOptionPair::Both(value, std::ptr::read(second));
                std::ptr::write(self, both);
            },
        }
    }

    /// AI生成：设置或添加第二个值
    ///
    /// # 行为
    /// - 在`Both`/`Second`状态：直接覆盖第二个值
    /// - 在`First`状态：转换为`Both`状态
    #[inline]
    pub fn set_second(&mut self, value: T2) {
        match self {
            ValidOptionPair::Both(_, second) | ValidOptionPair::Second(second) => {
                *second = value;
            }
            ValidOptionPair::First(first) => unsafe {
                let both = ValidOptionPair::Both(std::ptr::read(first), value);
                std::ptr::write(self, both);
            },
        }
    }
}

impl<T> ValidOptionPair<T, T> {
    /// AI生成：向实例添加值（当T1==T2时可用）
    ///
    /// # 行为
    /// - 在`First`状态：转换为`Both`并添加为第二值
    /// - 在`Second`状态：转换为`Both`并添加为第一值
    /// - 在`Both`状态：返回传入的值(`Some`)
    #[inline]
    pub fn push(&mut self, value: T) -> Option<T> {
        match self {
            ValidOptionPair::First(first) => unsafe {
                let both = ValidOptionPair::Both(std::ptr::read(first), value);
                std::ptr::write(self, both);
                None
            },
            ValidOptionPair::Second(second) => unsafe {
                let both = ValidOptionPair::Both(value, std::ptr::read(second));
                std::ptr::write(self, both);
                None
            },
            ValidOptionPair::Both(_, _) => Some(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// AI生成：基本功能测试
    #[test]
    fn test_valid_option_pair() {
        let mut a = ValidOptionPair::new_both(1, 2);
        let b = a.take_first();
        println!("a: {a:?}, b: {b:?}");
    }
}
