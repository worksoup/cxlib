/// AI生成：表示三选一值的枚举容器
///
/// 该枚举有三种变体：
/// - `First(T1)`: 仅包含第一种类型的值
/// - `Second(T2)`: 仅包含第二种类型的值
/// - `Last(T3)`: 仅包含第三种类型的值
///
/// 提供丰富的访问和操作方法，支持类型安全的值操作
#[derive(Debug)]
pub enum TripleVariant<T1, T2, T3> {
    /// AI生成：包含第一种类型值的变体
    First(T1),
    /// AI生成：包含第二种类型值的变体
    Second(T2),
    /// AI生成：包含第三种类型值的变体
    Last(T3),
}

impl<T1, T2, T3> TripleVariant<T1, T2, T3> {
    /// AI生成：消费实例返回First变体中的值（若非First则返回None）
    #[inline]
    pub fn into_first(self) -> Option<T1> {
        match self {
            TripleVariant::First(f) => Some(f),
            _ => None,
        }
    }

    /// AI生成：消费实例返回Second变体中的值（若非Second则返回None）
    #[inline]
    pub fn into_second(self) -> Option<T2> {
        match self {
            TripleVariant::Second(s) => Some(s),
            _ => None,
        }
    }

    /// AI生成：消费实例返回Last变体中的值（若非Last则返回None）
    #[inline]
    pub fn into_last(self) -> Option<T3> {
        match self {
            TripleVariant::Last(l) => Some(l),
            _ => None,
        }
    }

    /// AI生成：获取First变体中值的引用（若非First则返回None）
    #[inline]
    pub fn first(&self) -> Option<&T1> {
        match self {
            TripleVariant::First(f) => Some(f),
            _ => None,
        }
    }

    /// AI生成：获取Second变体中值的引用（若非Second则返回None）
    #[inline]
    pub fn second(&self) -> Option<&T2> {
        match self {
            TripleVariant::Second(s) => Some(s),
            _ => None,
        }
    }

    /// AI生成：获取Last变体中值的引用（若非Last则返回None）
    #[inline]
    pub fn last(&self) -> Option<&T3> {
        match self {
            TripleVariant::Last(l) => Some(l),
            _ => None,
        }
    }

    /// AI生成：检查当前是否为First变体
    #[inline]
    pub fn is_first(&self) -> bool {
        matches!(self, TripleVariant::First(_))
    }

    /// AI生成：检查当前是否为Second变体
    #[inline]
    pub fn is_second(&self) -> bool {
        matches!(self, TripleVariant::Second(_))
    }

    /// AI生成：检查当前是否为Last变体
    #[inline]
    pub fn is_last(&self) -> bool {
        matches!(self, TripleVariant::Last(_))
    }

    /// AI生成：映射First变体中的值
    ///
    /// 仅当当前为First变体时应用转换函数，其他变体保持不变
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

    /// AI生成：映射Second变体中的值
    ///
    /// 仅当当前为Second变体时应用转换函数，其他变体保持不变
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

    /// AI生成：映射Last变体中的值
    ///
    /// 仅当当前为Last变体时应用转换函数，其他变体保持不变
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

    /// AI生成：设置First变体的值
    ///
    /// 仅当当前已经是First变体时才会更新值
    /// 非First变体调用此方法无任何效果
    #[inline]
    pub fn set_first(&mut self, value: T1) {
        if let TripleVariant::First(_) = self {
            *self = TripleVariant::First(value);
        }
    }

    /// AI生成：设置Second变体的值
    ///
    /// 仅当当前已经是Second变体时才会更新值
    /// 非Second变体调用此方法无任何效果
    #[inline]
    pub fn set_second(&mut self, value: T2) {
        if let TripleVariant::Second(_) = self {
            *self = TripleVariant::Second(value);
        }
    }

    /// AI生成：设置Last变体的值
    ///
    /// 仅当当前已经是Last变体时才会更新值
    /// 非Last变体调用此方法无任何效果
    #[inline]
    pub fn set_last(&mut self, value: T3) {
        if let TripleVariant::Last(_) = self {
            *self = TripleVariant::Last(value);
        }
    }
}
