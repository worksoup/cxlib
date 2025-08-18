use cxlib_types::RawSign;

pub trait AsRaw {
    /// 获取对原始签到类型的引用。
    /// [`RawSign`] 的各字段均为 `pub`,
    /// 故可以通过本函数获取一些签到通用的信息。
    fn as_inner(&self) -> &RawSign;
}

impl AsRaw for RawSign {
    #[inline]
    fn as_inner(&self) -> &RawSign {
        self
    }
}
