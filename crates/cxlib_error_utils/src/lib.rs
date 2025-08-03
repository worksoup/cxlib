/// 是否为致命错误。
///
/// 在程序中经常存在一些循环，循环中的语句可能产生错误。
/// 我们不知道究竟该直接返回还是打个日志。
///
/// 所以可以靠该特型进行区分，如果是致命错误则返回，不是则忽略。
///
/// 事实上，这里的“致命错误”实质上是循环中将会稳定复现的错误。
/// 比如网络环境错误等，整个循环周期内都稳定产生错误，此时应当视为
/// 致命错误，避免浪费时间。
pub trait MaybeFatalError {
    /// 程序运行周期内可能长时间稳定产生的错误，应当视为致命错误，避免浪费时间。
    fn is_fatal(&self) -> bool;
}

#[inline]
#[track_caller]
pub fn log_none<T>(e: impl std::fmt::Debug) -> Option<T> {
    let caller = core::panic::Location::caller();
    log::warn!("{caller}: {e:?}, 将使用空值。",);
    None
}

#[inline]
#[track_caller]
pub fn log_panic<T>(e: impl std::fmt::Debug) -> T {
    let caller = core::panic::Location::caller();
    log::error!("{caller}: {e:?}.");
    panic!();
}

#[inline]
#[track_caller]
pub fn log_default<T: Default>(e: impl std::fmt::Debug) -> T {
    let caller = core::panic::Location::caller();
    log::warn!("{caller}: {e:?}, 将使用默认值。",);
    T::default()
}

pub trait CxlibResultUtils<T, E> {
    fn log_ignore(self);
    fn log_unwrap(self) -> T;
    fn log_unwrap_or_default(self) -> T
    where
        T: Default;
    fn log_ok(self) -> Option<T>;
    fn ok_or_else<F: FnOnce(E) -> Option<T>>(self, f: F) -> Option<T>;
}
impl<T, E: std::fmt::Debug> CxlibResultUtils<T, E> for Result<T, E> {
    #[inline]
    #[track_caller]
    fn log_unwrap(self) -> T {
        self.unwrap_or_else(log_panic)
    }

    #[inline]
    #[track_caller]
    fn log_unwrap_or_default(self) -> T
    where
        T: Default,
    {
        self.unwrap_or_else(log_default)
    }

    #[inline]
    #[track_caller]
    fn log_ok(self) -> Option<T> {
        self.ok_or_else(log_none)
    }

    #[inline]
    #[track_caller]
    fn log_ignore(self) {
        let caller = core::panic::Location::caller();
        match self {
            Ok(_) => {log::debug!("{caller}: 值已被忽略。",)},

            Err(e) => log::warn!("{caller}: 忽略错误：{e:?}。",),
        }
    }

    #[inline]
    #[track_caller]
    fn ok_or_else<F: FnOnce(E) -> Option<T>>(self, f: F) -> Option<T> {
        match self {
            Ok(x) => Some(x),
            Err(e) => f(e),
        }
    }
}
