//! 提供带时间测量的函数执行工具集，仅供 Debug 使用。
//!
//! *以下内容由 AI 生成。*
//!
//! 此模块包含了一系列在 debug 模式下测量函数执行时间的实用函数，
//! 在 release 构建中则自动退化为无测量开销的标准执行。
//!
//! ## 核心功能
//!
//! - 测量闭包执行时间并返回结果
//! - 可选输出耗时信息
//! - 在 release 构建中自动编译为无开销版本
//! - 支持任意返回类型的闭包
//!
//! ## 使用示例
//!
//! ```rust
//! let result = cx_debug_utils::time_it_and_print_result(|| {
//!     // 需要测量的代码逻辑
//!     42 // 返回值
//! });
//! ```
//!
//! 在 debug 模式下会输出："cost Xms."，在 release 模式下无输出
//!
//! ## 设计特点
//!
//! - **条件编译**：仅在 `debug_assertions` 启用时包含计时逻辑
//! - **零开销**：release 构建中完全消除测量开销
//! - **内联优化**：通过 `#[inline(always)]` 确保无额外调用开销
//! - **类型安全**：泛型实现支持任意返回类型

/// *以下内容由 AI 生成。*
/// 
/// 运行闭包并计时，输出计时结果，返回运行结果。
///
/// 在 debug 模式下：测量闭包执行时间，打印耗时（毫秒），返回闭包结果  
/// 在 release 模式下：等同于直接执行闭包  
///
/// # 参数
/// - `f`: 要执行的闭包
///
/// # 返回
/// 闭包的执行结果  
///
/// # 示例
/// ```rust
/// let value = cx_debug_utils::time_it_and_print_result(|| {
///     // 需要计时的操作
///     42
/// });
/// ```
pub fn time_it_and_print_result<R, F: FnOnce() -> R>(f: F) -> R {
    print_timed_result(time_it(f))
}

/// *以下内容由 AI 生成。*
/// 
/// 运行闭包并计时，返回运行结果与耗时
///
/// **调试模式专用**:  
/// - 返回元组 `(闭包结果, 耗时毫秒)`  
///
/// **发布模式行为**:  
/// - 直接返回闭包结果（无耗时测量）
#[inline(always)]
#[cfg(not(debug_assertions))]
pub fn time_it<R, F: FnOnce() -> R>(f: F) -> R {
    f()
}

/// *以下内容由 AI 生成。*
/// 
/// 运行闭包并计时，返回运行结果与耗时
///
/// **调试模式专用**:  
/// - 返回元组 `(闭包结果, 耗时毫秒)`  
///
/// **发布模式行为**:  
/// - 直接返回闭包结果（无耗时测量）
#[inline(always)]
#[cfg(debug_assertions)]
pub fn time_it<R, F: FnOnce() -> R>(f: F) -> (R, u128) {
    {
        let start = std::time::Instant::now();
        let r = f();
        let elapsed = start.elapsed();
        (r, elapsed.as_millis())
    }
}

/// *以下内容由 AI 生成。*
/// 
/// 处理计时结果并打印耗时信息
///
/// **调试模式专用**:  
/// - 输入为 `(结果, 耗时)` 元组  
/// - 打印耗时（毫秒）  
/// - 返回结果部分  
///
/// **发布模式行为**:  
/// - 直接原样返回输入值
#[inline(always)]
#[cfg(debug_assertions)]
pub fn print_timed_result<T>(result: (T, u128)) -> T {
    println!("cost {}ms.", result.1);
    result.0
}

/// *以下内容由 AI 生成。*
/// 
/// 处理计时结果并打印耗时信息
///
/// **调试模式专用**:  
/// - 输入为 `(结果, 耗时)` 元组  
/// - 打印耗时（毫秒）  
/// - 返回结果部分  
///
/// **发布模式行为**:  
/// - 直接原样返回输入值
#[inline(always)]
#[cfg(not(debug_assertions))]
pub fn print_timed_result<T>(result: T) -> T {
    result
}
