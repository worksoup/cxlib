use std::ops::Deref;

/// AI生成：表示包含主值及可选附加值的枚举类型
///
/// 该枚举有两种变体：
/// - `One(T1)`：仅包含主值
/// - `WithOther(T1, T2)`：同时包含主值和附加值
pub enum OneWithOption<T1, T2> {
    /// AI生成：仅包含主值的变体
    One(T1),
    /// AI生成：包含主值和附加值的变体
    WithOther(T1, T2),
}

impl<T1, T2> Deref for OneWithOption<T1, T2> {
    type Target = T1;

    /// AI生成：解引用到主值的不可变引用
    fn deref(&self) -> &Self::Target {
        self.one()
    }
}

impl<T1, T2> From<T1> for OneWithOption<T1, T2> {
    /// AI生成：从主值创建`One`变体
    #[inline]
    fn from(value: T1) -> Self {
        Self::One(value)
    }
}

impl<T1, T2> From<(T1, Option<T2>)> for OneWithOption<T1, T2> {
    /// AI生成：从元组(T1, Option<T2>)创建实例
    #[inline]
    fn from(value: (T1, Option<T2>)) -> Self {
        Self::from_tuple(value)
    }
}

impl<T1, T2> From<(T1, T2)> for OneWithOption<T1, T2> {
    /// AI生成：从元组(T1, T2)创建`WithOther`变体
    #[inline]
    fn from((one, other): (T1, T2)) -> Self {
        Self::WithOther(one, other)
    }
}

impl<T1, T2> OneWithOption<T1, T2> {
    /// AI生成：根据主值和可选附加值创建新实例
    ///
    /// - 当`t2`为`Some`时返回`WithOther`变体
    /// - 当`t2`为`None`时返回`One`变体
    #[inline]
    pub fn new(t1: T1, t2: Option<T2>) -> Self {
        if let Some(t2) = t2 {
            Self::WithOther(t1, t2)
        } else {
            Self::One(t1)
        }
    }

    /// AI生成：创建仅包含主值的`One`变体
    #[inline]
    pub fn new_one(t1: T1) -> Self {
        Self::One(t1)
    }

    /// AI生成：从元组(T1, Option<T2>)创建实例
    ///
    /// 功能等同于`new`方法
    #[inline]
    pub fn from_tuple(t: (T1, Option<T2>)) -> Self {
        match t {
            (t1, Some(t2)) => Self::WithOther(t1, t2),
            (t1, None) => Self::One(t1),
        }
    }

    /// AI生成：获取包含内部值引用的新实例
    #[inline]
    pub fn as_ref(&self) -> OneWithOption<&T1, &T2> {
        match self {
            OneWithOption::One(one) => OneWithOption::One(one),
            OneWithOption::WithOther(one, other) => OneWithOption::WithOther(one, other),
        }
    }

    /// AI生成：获取包含内部值可变引用的新实例
    #[inline]
    pub fn as_mut(&mut self) -> OneWithOption<&mut T1, &mut T2> {
        match self {
            OneWithOption::One(one) => OneWithOption::One(one),
            OneWithOption::WithOther(one, other) => OneWithOption::WithOther(one, other),
        }
    }

    /// AI生成：取出附加值并转换为`One`变体
    ///
    /// 返回被移除的附加值（如果存在）
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

    /// AI生成：消费实例返回主值
    #[inline]
    pub fn into_one(self) -> T1 {
        match self {
            OneWithOption::One(one) | OneWithOption::WithOther(one, _) => one,
        }
    }

    /// AI生成：消费实例返回附加值（如果存在）
    #[inline]
    pub fn into_other(self) -> Option<T2> {
        match self {
            OneWithOption::One(_) => None,
            OneWithOption::WithOther(_, other) => Some(other),
        }
    }

    /// AI生成：消费实例返回元组(T1, Option<T2>)
    #[inline]
    pub fn into_tuple(self) -> (T1, Option<T2>) {
        match self {
            OneWithOption::One(one) => (one, None),
            OneWithOption::WithOther(one, other) => (one, Some(other)),
        }
    }

    /// AI生成：获取主值的不可变引用
    #[inline]
    pub fn one(&self) -> &T1 {
        self.as_ref().into_one()
    }

    /// AI生成：获取附加值的不可变引用（如果存在）
    #[inline]
    pub fn other(&self) -> Option<&T2> {
        self.as_ref().into_other()
    }

    /// AI生成：获取主值和附加值的引用元组
    #[inline]
    pub fn tuple(&self) -> (&T1, Option<&T2>) {
        self.as_ref().into_tuple()
    }

    /// AI生成：获取主值的可变引用
    #[inline]
    pub fn one_mut(&mut self) -> &mut T1 {
        self.as_mut().into_one()
    }

    /// AI生成：获取附加值的可变引用（如果存在）
    #[inline]
    pub fn other_mut(&mut self) -> Option<&mut T2> {
        self.as_mut().into_other()
    }

    /// AI生成：获取主值和附加值的可变引用元组
    #[inline]
    pub fn tuple_mut(&mut self) -> (&mut T1, Option<&mut T2>) {
        self.as_mut().into_tuple()
    }

    /// AI生成：检查是否存在附加值
    #[inline]
    pub fn has_other(&self) -> bool {
        match self {
            OneWithOption::One(_) => false,
            OneWithOption::WithOther(_, _) => true,
        }
    }

    /// AI生成：对主值和附加值进行映射转换
    ///
    /// 使用两个闭包分别处理：
    /// - `ff`: 处理主值`T1 -> B1`
    /// - `sf`: 处理附加值`T2 -> B2`
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

    /// AI生成：仅映射主值，保持附加值不变
    #[inline]
    pub fn map_one<B, F>(self, f: F) -> OneWithOption<B, T2>
    where
        Self: Sized,
        F: FnMut(T1) -> B,
    {
        self.map(f, |s| s)
    }

    /// AI生成：仅映射附加值，保持主值不变
    #[inline]
    pub fn map_other<B, F>(self, f: F) -> OneWithOption<T1, B>
    where
        Self: Sized,
        F: FnMut(T2) -> B,
    {
        self.map(|f| f, f)
    }

    /// AI生成：尝试添加附加值
    ///
    /// 若当前为`One`变体，转换为`WithOther`并返回`None`
    /// 若当前已有附加值，返回传入的附加值（不改变自身状态）
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

    /// AI生成：设置主值（所有变体均适用）
    #[inline]
    pub fn set_one(&mut self, value: T1) {
        match self {
            OneWithOption::One(one) | OneWithOption::WithOther(one, _) => {
                *one = value;
            }
        }
    }

    /// AI生成：设置附加值（仅当`WithOther`变体时生效）
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
