
/// 运行闭包并计时，输出计时结果，返回运行结果。
///
/// 仅在 debug_assertions 下作用，否则相当于运行闭包。
pub fn time_it_and_print_result<R, F: FnOnce() -> R>(f: F) -> R {
    print_timed_result(time_it(f))
}
/// 运行闭包并计时，返回运行结果与计时结果。
///
/// 仅在 debug_assertions 下作用，否则相当于运行闭包。
#[inline(always)]
#[cfg(not(debug_assertions))]
pub fn time_it<R, F: FnOnce() -> R>(f: F) -> R {
    f()
}
/// 运行闭包并计时，返回运行结果与计时结果。
///
/// 仅在 debug_assertions 下作用，否则相当于运行闭包。
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
/// 与 time_it 一起使用，输出计时结果，返回运行结果与。
///
/// 仅在 debug_assertions 下作用，否则仅仅传递数据。
#[inline(always)]
#[cfg(debug_assertions)]
pub fn print_timed_result<T>(result: (T, u128)) -> T {
    println!("cost {}ms.", result.1);
    result.0
}
/// 与 time_it 一起使用，输出计时结果，返回运行结果与。
///
/// 仅在 debug_assertions 下作用，否则仅仅传递数据。
#[inline(always)]
#[cfg(not(debug_assertions))]
pub fn print_timed_result<T>(result: T) -> T {
    result
}