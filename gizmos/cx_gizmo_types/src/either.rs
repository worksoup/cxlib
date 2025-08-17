/// `Either` 枚举用于表示两种互斥的可能性之一：
/// 包含类型为 `T1` 的值或类型为 `T2` 的值。
///
/// 该枚举类似于标准库的 [`Result`] 和 [`Option`]，但适用于任意两种类型，
/// 为处理两种可能类型提供了一种统一的方式。
///
/// # 示例
/// ```
/// use your_crate::Either;
///
/// // 处理整数或字符串
/// let int_val: Either<i32, &str> = Either::A(42);
/// let str_val: Either<i32, &str> = Either::B("hello");
///
/// // 使用模式匹配处理不同情况
/// match int_val {
///     Either::A(n) => println!("整数: {}", n),
///     Either::B(s) => println!("字符串: {}", s),
/// }
/// ```
///
/// [`Result`]: std::result::Result
/// [`Option`]: std::option::Option
#[derive(Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum Either<T1, T2> {
    /// 第一种可能值（通常表示主类型）
    A(T1),
    /// 第二种可能值（通常表示替代类型）
    B(T2),
}

/////////////////////////////////////////////////////////////////////////////
// Type implementation
/////////////////////////////////////////////////////////////////////////////
impl<T1, T2> Either<T1, T2> {
    /// 转换为包含内部值引用的新 `Either` 实例
    ///
    /// 该方法保留原始枚举变体，但将其内部值转换为不可变引用。
    ///
    /// # 示例
    /// ```
    /// use your_crate::Either;
    ///
    /// let value = Either::A(vec![1, 2, 3]);
    /// let ref_value = value.as_ref();
    ///
    /// // ref_value 现在是 Either::A(&Vec<i32>) 类型
    /// match ref_value {
    ///     Either::A(v) => println!("长度: {}", v.len()),
    ///     Either::B(_) => unreachable!(),
    /// }
    /// ```
    #[inline]
    pub const fn as_ref(&self) -> Either<&T1, &T2> {
        match *self {
            Self::A(ref x) => Either::A(x),
            Self::B(ref x) => Either::B(x),
        }
    }

    /// 转换为包含内部值可变引用的新 `Either` 实例
    ///
    /// 该方法保留原始枚举变体，但将其内部值转换为可变引用。
    ///
    /// # 示例
    /// ```
    /// use your_crate::Either;
    ///
    /// let mut value = Either::B(String::from("hello"));
    /// {
    ///     let mut_ref = value.as_mut();
    ///     if let Either::B(s) = mut_ref {
    ///         s.push_str(" world!");
    ///     }
    /// }
    ///
    /// assert_eq!(value, Either::B("hello world!".to_string()));
    /// ```
    #[inline]
    pub const fn as_mut(&mut self) -> Either<&mut T1, &mut T2> {
        match *self {
            Self::A(ref mut x) => Either::A(x),
            Self::B(ref mut x) => Either::B(x),
        }
    }
}

/////////////////////////////////////////////////////////////////////////////
// Trait implementations
/////////////////////////////////////////////////////////////////////////////

/// 实现 `Clone` trait，要求内部类型 `T1` 和 `T2` 均实现 `Clone`
///
/// 克隆操作会根据当前变体深度克隆内部值：
/// - `A` 变体：克隆 `T1` 类型值
/// - `B` 变体：克隆 `T2` 类型值
impl<T1, T2> Clone for Either<T1, T2>
where
    T1: Clone,
    T2: Clone,
{
    /// 创建当前 `Either` 实例的深度副本
    ///
    /// # 示例
    /// ```
    /// use your_crate::Either;
    ///
    /// let original = Either::A(Box::new(10));
    /// let cloned = original.clone();
    ///
    /// // 两个实例包含不同的指针
    /// assert_eq!(original, cloned);
    /// assert!(!std::ptr::eq(&original, &cloned));
    /// ```
    #[inline]
    fn clone(&self) -> Self {
        match self {
            Self::A(x) => Self::A(x.clone()),
            Self::B(x) => Self::B(x.clone()),
        }
    }

    /// 复用现有分配，从源实例克隆数据
    ///
    /// 当两个实例为相同变体时，复用内部分配；
    /// 当变体不同时，执行完整克隆覆盖目标。
    ///
    /// # 示例
    /// ```
    /// use your_crate::Either;
    ///
    /// let src = Either::A(String::from("source"));
    /// let mut dst = Either::A(String::with_capacity(20));
    ///
    /// dst.clone_from(&src);
    /// assert_eq!(dst, Either::A("source".to_string()));
    /// ```
    #[inline]
    fn clone_from(&mut self, source: &Self) {
        match (self, source) {
            (Self::A(to), Self::A(from)) => to.clone_from(from),
            (Self::B(to), Self::B(from)) => to.clone_from(from),
            (to, from) => *to = from.clone(),
        }
    }
}
