//! *本文档由 AI 生成。*
//!
//! 提供零成本透明包装类型及相关工具
//!
//! 本模块定义了一个透明内存布局的智能包装类型 [`RefWrapper`]，
//! 用于在不改变内存表示的情况下为类型添加额外功能。特别提供了
//! 单元类型 [`Unit`] 的包装实现，用于需要引用语义的空值场景。
//!
//! # 设计特点
//! 1. **零开销抽象**：`#[repr(transparent)]` 保证内存布局与内层类型完全一致
//! 2. **引用透明**：自动实现 `Deref` 和 `AsRef` 保持原始类型的访问语义
//! 3. **单元类型优化**：为单元类型 `()` 提供专用包装和常量实例

use std::ops::Deref;

/// *本文档由 AI 生成。*
///
/// 单元包装类型别名（仅用于语义指示）
///
/// 等同于 [`Unit`]，主要作为类型标记表示"无需返回值"的语义场景。
pub type DoesntMatter = Unit;

/// *本文档由 AI 生成。*
///
/// 单元类型的智能包装
///
/// 包装 `()` 类型并提供引用语义，常用于以下场景：
/// - 需要引用空值的 API 设计
/// - 类型系统中标记"无返回值"的状态
/// - 作为占位符实现泛型约束
///
/// # 内存表示
/// 内存布局与 `()` 完全一致，在运行时无任何额外开销。
pub type Unit = RefWrapper<()>;

/// *本文档由 AI 生成。*
///
/// 单元类型的全局常量实例
///
/// 用法示例：
/// ```
/// # use your_crate::UNIT;
/// fn process() -> &'static Unit {
///     &UNIT  // 返回单元包装的静态引用
/// }
/// ```
pub const UNIT: Unit = Unit::__;

impl Unit {
    /// *本文档由 AI 生成。*
    ///
    /// 构造单元包装的内部常量（仅供常量初始化使用）
    const __: Self = RefWrapper(());
}

/// *本文档由 AI 生成。*
///
/// 透明内存布局的智能包装器
///
/// 在保持原始类型内存布局的前提下，为其添加智能指针行为。
/// 通过自动实现 `Deref` 和 `AsRef` 保持原始类型的访问接口。
///
/// # 泛型参数
/// - `T`：被包装的类型，支持任意大小类型（包括动态大小类型）
///
/// # 内存安全
/// 因 `#[repr(transparent)]` 保证，此类型可在 FFI 中安全替代原始类型。
///
/// # 示例
/// ```
/// # use your_crate::RefWrapper;
/// let num = RefWrapper(42);
/// assert_eq!(*num, 42);       // 通过 Deref 访问
/// assert_eq!(num.as_ref(), &42); // 通过 AsRef 访问
/// ```
#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialOrd, PartialEq, Default)]
#[repr(transparent)]
pub struct RefWrapper<T: ?Sized>(pub T);

impl<T: ?Sized, B: ?Sized + std::borrow::Borrow<T>> AsRef<T> for RefWrapper<B> {
    /// *本文档由 AI 生成。*
    ///
    /// 获取内部值的不可变引用
    ///
    /// 实际调用内层类型的 `Borrow::borrow` 实现，
    /// 支持从包装类型安全地借用原始类型引用。
    #[inline]
    fn as_ref(&self) -> &T {
        self.0.borrow()
    }
}

impl<T: ?Sized> Deref for RefWrapper<T> {
    type Target = T;

    /// *本文档由 AI 生成。*
    ///
    /// 自动解引用到内部类型
    ///
    /// 允许直接使用原始类型的方法和字段：
    /// ```
    /// # use your_crate::RefWrapper;
    /// let text = RefWrapper("hello");
    /// assert_eq!(text.len(), 5);  // 直接调用 &str 的方法
    /// ```
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
