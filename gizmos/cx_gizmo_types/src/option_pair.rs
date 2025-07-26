use crate::{OneWithOption, ValidOptionPair};

/// AI生成：表示两个可选值的组合容器
///
/// 该结构体包含两个可选字段：
/// - `first`: 第一个可选值
/// - `second`: 第二个可选值
///
/// 支持从多种元组类型转换，提供丰富的访问和操作方法
#[derive(Debug)]
pub struct OptionPair<T1, T2> {
    /// AI生成：第一个可选值
    first: Option<T1>,
    /// AI生成：第二个可选值
    second: Option<T2>,
}

impl<T1, T2> From<(Option<T1>, Option<T2>)> for OptionPair<T1, T2> {
    /// AI生成：从(Option<T1>, Option<T2>)元组创建实例
    #[inline]
    fn from(v: (Option<T1>, Option<T2>)) -> Self {
        Self::from_tuple(v)
    }
}

impl<T1, T2> From<(T1, Option<T2>)> for OptionPair<T1, T2> {
    /// AI生成：从(T1, Option<T2>)元组创建实例
    #[inline]
    fn from((first, second): (T1, Option<T2>)) -> Self {
        Self {
            first: Some(first),
            second,
        }
    }
}

impl<T1, T2> From<(Option<T1>, T2)> for OptionPair<T1, T2> {
    /// AI生成：从(Option<T1>, T2)元组创建实例
    #[inline]
    fn from((first, second): (Option<T1>, T2)) -> Self {
        Self {
            first,
            second: Some(second),
        }
    }
}

impl<T1, T2> From<(T1, T2)> for OptionPair<T1, T2> {
    /// AI生成：从(T1, T2)元组创建实例（两个值都存在）
    #[inline]
    fn from((first, second): (T1, T2)) -> Self {
        Self {
            first: Some(first),
            second: Some(second),
        }
    }
}

impl<T1, T2> From<ValidOptionPair<T1, T2>> for OptionPair<T1, T2> {
    /// AI生成：从ValidOptionPair转换
    #[inline]
    fn from(value: ValidOptionPair<T1, T2>) -> Self {
        Self::from_tuple(value.into_tuple())
    }
}

impl<T1, T2> From<OneWithOption<T1, T2>> for OptionPair<T1, T2> {
    /// AI生成：从OneWithOption转换
    #[inline]
    fn from(value: OneWithOption<T1, T2>) -> Self {
        value.into_tuple().into()
    }
}

impl<T1, T2> OptionPair<T1, T2> {
    /// AI生成：创建两个值都为None的空实例
    #[inline]
    pub fn new_none() -> Self {
        Self {
            first: None,
            second: None,
        }
    }

    /// AI生成：创建只有第一个值的实例
    #[inline]
    pub fn new_first(f: T1) -> Self {
        Self {
            first: Some(f),
            second: None,
        }
    }

    /// AI生成：创建只有第二个值的实例
    #[inline]
    pub fn new_second(s: T2) -> Self {
        Self {
            first: None,
            second: Some(s),
        }
    }

    /// AI生成：创建包含两个值的实例
    #[inline]
    pub fn new_both(f: T1, s: T2) -> Self {
        Self {
            first: Some(f),
            second: Some(s),
        }
    }

    /// AI生成：获取内部值的引用版本
    #[inline]
    pub fn as_ref(&self) -> OptionPair<&T1, &T2> {
        OptionPair {
            first: self.first.as_ref(),
            second: self.second.as_ref(),
        }
    }

    /// AI生成：获取内部值的可变引用版本
    #[inline]
    pub fn as_mut(&mut self) -> OptionPair<&mut T1, &mut T2> {
        OptionPair {
            first: self.first.as_mut(),
            second: self.second.as_mut(),
        }
    }

    /// AI生成：从(Option<T1>, Option<T2>)元组创建实例
    #[inline]
    pub fn from_tuple((first, second): (Option<T1>, Option<T2>)) -> Self {
        Self { first, second }
    }

    /// AI生成：取出并返回第一个值（原位置置为None）
    #[inline]
    pub fn take_first(&mut self) -> Option<T1> {
        self.first.take()
    }

    /// AI生成：取出并返回第二个值（原位置置为None）
    #[inline]
    pub fn take_second(&mut self) -> Option<T2> {
        self.second.take()
    }

    /// AI生成：消费实例返回第一个值
    #[inline]
    pub fn into_first(self) -> Option<T1> {
        self.first
    }

    /// AI生成：消费实例返回第二个值
    #[inline]
    pub fn into_second(self) -> Option<T2> {
        self.second
    }

    /// AI生成：消费实例返回两个值（仅当两者都存在时返回Some）
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

    /// AI生成：消费实例返回元组(Option<T1>, Option<T2>)
    #[inline]
    pub fn into_tuple(self) -> (Option<T1>, Option<T2>) {
        (self.first, self.second)
    }

    /// AI生成：获取第一个值的引用
    #[inline]
    pub fn first(&self) -> Option<&T1> {
        self.as_ref().into_first()
    }

    /// AI生成：获取第二个值的引用
    #[inline]
    pub fn second(&self) -> Option<&T2> {
        self.as_ref().into_second()
    }

    /// AI生成：同时获取两个值的引用（仅当两者都存在时返回Some）
    #[inline]
    pub fn both(&self) -> Option<(&T1, &T2)> {
        self.as_ref().into_both()
    }

    /// AI生成：获取两个值的引用元组
    #[inline]
    pub fn tuple(&self) -> (Option<&T1>, Option<&T2>) {
        self.as_ref().into_tuple()
    }

    /// AI生成：获取第一个值的可变引用
    #[inline]
    pub fn first_mut(&mut self) -> Option<&mut T1> {
        self.as_mut().into_first()
    }

    /// AI生成：获取第二个值的可变引用
    #[inline]
    pub fn second_mut(&mut self) -> Option<&mut T2> {
        self.as_mut().into_second()
    }

    /// AI生成：同时获取两个值的可变引用（仅当两者都存在时返回Some）
    #[inline]
    pub fn both_mut(&mut self) -> Option<(&mut T1, &mut T2)> {
        self.as_mut().into_both()
    }

    /// AI生成：获取两个值的可变引用元组
    #[inline]
    pub fn tuple_mut(&mut self) -> (Option<&mut T1>, Option<&mut T2>) {
        self.as_mut().into_tuple()
    }

    /// AI生成：检查是否两个值都不存在
    #[inline]
    pub fn is_none(&self) -> bool {
        self.first.is_none() && self.second.is_none()
    }

    /// AI生成：检查是否只有第一个值存在
    #[inline]
    pub fn is_first(&self) -> bool {
        self.first.is_some() && self.second.is_none()
    }

    /// AI生成：检查是否只有第二个值存在
    #[inline]
    pub fn is_second(&self) -> bool {
        self.second.is_some() && self.first.is_none()
    }

    /// AI生成：检查是否两个值都存在
    #[inline]
    pub fn is_both(&self) -> bool {
        self.first.is_some() && self.second.is_some()
    }

    /// AI生成：检查第一个值是否存在
    #[inline]
    pub fn has_first(&self) -> bool {
        self.first.is_some()
    }

    /// AI生成：检查第二个值是否存在
    #[inline]
    pub fn has_second(&self) -> bool {
        self.second.is_some()
    }

    /// AI生成：对两个值进行映射转换
    ///
    /// 使用两个闭包分别处理：
    /// - `ff`: 处理第一个值 `T1 -> B1`
    /// - `sf`: 处理第二个值 `T2 -> B2`
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

    /// AI生成：仅映射第一个值，保持第二个值不变
    #[inline]
    pub fn map_first<B, FF>(self, ff: FF) -> OptionPair<B, T2>
    where
        Self: Sized,
        FF: FnMut(T1) -> B,
    {
        self.map(ff, |s| s)
    }

    /// AI生成：仅映射第二个值，保持第一个值不变
    #[inline]
    pub fn map_second<B, F>(self, f: F) -> OptionPair<T1, B>
    where
        Self: Sized,
        F: FnMut(T2) -> B,
    {
        self.map(|f| f, f)
    }

    /// AI生成：设置第一个值并返回旧值
    #[inline]
    pub fn push_first(&mut self, value: T1) -> Option<T1> {
        self.first.replace(value)
    }

    /// AI生成：设置第二个值并返回旧值
    #[inline]
    pub fn push_second(&mut self, value: T2) -> Option<T2> {
        self.second.replace(value)
    }

    /// AI生成：无条件设置第一个值（覆盖原有值）
    #[inline]
    pub fn set_first(&mut self, value: T1) {
        self.first = Some(value);
    }

    /// AI生成：无条件设置第二个值（覆盖原有值）
    #[inline]
    pub fn set_second(&mut self, value: T2) {
        self.second = Some(value);
    }
}

impl<T> OptionPair<T, T> {
    /// AI生成：智能添加值（优先填充空位）
    ///
    /// 添加策略：
    /// - 如果第一个值不存在，添加到第一个位置
    /// - 如果第二个值不存在，添加到第二个位置
    /// - 如果两个值都已存在，返回传入的值
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
