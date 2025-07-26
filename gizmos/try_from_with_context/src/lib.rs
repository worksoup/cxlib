//! *本文档由 AI 生成。*
//!
//! 提供带有上下文的类型转换工具
//!
//! 本模块定义了可以通过上下文进行类型转换的 trait，特别适用于需要额外信息
//!（如解析器状态、环境变量等）的字符串解析场景。
//!
//! 主要包含三个 trait：
//! - [`TryFromWithContext`]：带上下文的类型转换
//! - [`FromStrWithContext`]：带上下文的字符串解析
//! - [`ParseWith`]：为所有类型添加解析扩展方法

use std::borrow::Borrow;

/// *本文档由 AI 生成。*
///
/// 支持通过上下文进行类型转换的 trait
///
/// 类似于标准库的 `TryFrom`，但接受额外的上下文参数，适用于需要
/// 环境配置或解析规则的场景（如需要符号表的编程语言解析）。
///
/// # 泛型参数
/// - `T`：被转换的原始类型（如 `&str`）
///
/// # 关联类型
/// - `Err`：转换失败时的错误类型
/// - `Context`：上下文类型（支持动态大小类型）
///
/// # 生命周期
/// - `'cxt`：上下文的生命周期，确保上下文在转换期间有效
///
/// # 示例
/// 实现自定义解析器：
/// ```ignore
/// struct ParserContext {
///     symbols: HashMap<String, Value>,
/// }
///
/// impl TryFromWithContext<&str> for Expression {
///     type Err = ParseError;
///     type Context<'cxt> = ParserContext;
///     
///     fn try_from<'cxt>(
///         s: &str,
///         cxt: impl Borrow<Self::Context<'cxt>>
///     ) -> Result<Self, Self::Err> {
///         // 使用 cxt.borrow().symbols 解析表达式
///         # unimplemented!()
///     }
/// }
/// ```
pub trait TryFromWithContext<T>: Sized {
    type Err;
    type Context<'cxt>: ?Sized;

    /// *本文档由 AI 生成。*
    ///
    /// 执行带上下文的类型转换
    ///
    /// 参数 `cxt` 需实现 `Borrow<Self::Context>` 以支持灵活借用，
    /// 允许传递上下文引用或智能指针。
    fn try_from<'cxt, Cxt: Borrow<Self::Context<'cxt>>>(s: T, cxt: Cxt) -> Result<Self, Self::Err>;
}

/// *本文档由 AI 生成。*
///
/// 专为字符串解析设计的扩展 trait
///
/// 为所有实现了 `TryFromWithContext<&'a str>` 的类型自动提供 `from_str` 方法，
/// 简化字符串解析调用。
///
/// 本 trait 会自动为符合条件的类型实现，无需手动实现。
pub trait FromStrWithContext<'a>: TryFromWithContext<&'a str> {
    /// *本文档由 AI 生成。*
    ///
    /// 使用上下文解析字符串
    ///
    /// 调用方式：`Type::from_str("text", context)?`
    ///
    /// 实际委托给 `TryFromWithContext::try_from` 实现。
    fn from_str<'cxt, Cxt: Borrow<Self::Context<'cxt>>>(
        s: &'a str,
        cxt: Cxt,
    ) -> Result<Self, Self::Err> {
        TryFromWithContext::try_from(s, cxt)
    }
}

// 为所有实现 TryFromWithContext<&str> 的类型自动实现 FromStrWithContext
impl<'a, T: TryFromWithContext<&'a str>> FromStrWithContext<'a> for T {}

/// *本文档由 AI 生成。*
///
/// 为所有类型添加解析扩展方法
///
/// 通过扩展方法模式，为任意类型 `T` 添加 `parse_with` 方法，
/// 实现类似 `str::parse` 但支持上下文的解析功能。
///
/// # 用法示例
/// ```ignore
/// let expr = "x + 5".parse_with(&context)?;
/// ```
pub trait ParseWith: Sized {
    /// *本文档由 AI 生成。*
    ///
    /// 使用上下文解析当前值
    ///
    /// 调用方式：`input_str.parse_with(context)?`
    ///
    /// # 类型推断
    /// 通常需指定返回类型（通过 turbofish 或变量类型注解）：
    /// ```ignore
    /// let expr: Expression = text.parse_with(&ctx)?;
    /// // 或
    /// let expr = text.parse_with::<Expression, _>(&ctx)?;
    /// ```
    fn parse_with<'cxt, T, Cxt>(self, cxt: Cxt) -> Result<T, T::Err>
    where
        T: TryFromWithContext<Self>,
        Cxt: Borrow<T::Context<'cxt>>,
    {
        T::try_from(self, cxt)
    }
}

// 为所有类型自动实现 ParseWith，使其获得 parse_with 方法
impl<T> ParseWith for T {}
