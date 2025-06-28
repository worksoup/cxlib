pub mod crypto {
    //! PKCS#7 填充方案非标准实现
    //!
    //! 本模块提供了符合 PKCS#7 标准的填充方案实现，用于将数据填充至指定块大小的整数倍。
    //!
    //! ## PKCS#7 填充规则（非标准扩展版）
    //! - 填充单元大小小于一个 `usize`.
    //! - 如果数据长度不是块大小(BLOCK_SIZE)的整数倍，填充 `n` 个单元，每个单元值为 `n`
    //!   （`n` 是缺少的单元数）。
    //! - 如果数据长度恰好是块大小的整数倍，则填充一整块单元，每个单元值为块大小。
    //!
    //! ## 泛型参数要求
    //! - `BLOCK_SIZE` 为块大小，必须在 1-(2.pow(size_of::<T>())-1) 范围内。
    //! - `T` 为填充单元数据类型。标准 PKCS#7 要求为 u8 等单字节的数据类型，
    //!   本函数不作要求。但需保证：对于任意填充合法且允许通过字节直接初始化。
    //!
    //! ## 示例
    //! ```
    //! // 8字节块大小
    //! use std::cell::UnsafeCell;
    //! let data = b"hello";
    //! let padded = cx_enc_utils::crypto::pkcs7_pad::<8, u8>(data);
    //! println!("{padded:?}");
    //! assert_eq!(padded[0], [b'h', b'e', b'l', b'l', b'o', 3, 3, 3]);
    //! ```
    use std::marker::PhantomData;
    use std::mem::MaybeUninit;
    use std::{mem, ptr};

    /// PKCS#7 填充方案非标准实现
    ///
    /// ## PKCS#7 填充规则（非标准扩展版）
    /// - 填充单元大小小于一个 `usize`.
    /// - 如果数据长度不是块大小(BLOCK_SIZE)的整数倍，填充 `n` 个单元，每个单元值为 `n`
    ///   （`n` 是缺少的单元数）。
    /// - 如果数据长度恰好是块大小的整数倍，则填充一整块单元，每个单元值为块大小。
    ///
    /// ## 泛型参数要求
    /// - `BLOCK_SIZE` 为块大小，必须在 1-(2.pow(size_of::<T>())-1) 范围内。
    /// - `T` 为填充单元数据类型。标准 PKCS#7 要求为 u8 等单字节的数据类型，
    ///   本函数不作要求。但需保证：对于任意填充合法且允许通过字节直接初始化。
    ///
    /// ## 参数要求
    /// - `data` 为待填充数据。本函数会直接调用 `extend` 进行填充。
    ///
    /// ## 示例
    /// ```
    /// // 8字节块大小
    /// use std::cell::UnsafeCell;
    /// let data = b"hello";
    /// let padded = cx_enc_utils::crypto::pkcs7_pad::<8, u8>(data);
    /// println!("{padded:?}");
    /// assert_eq!(padded[0], [b'h', b'e', b'l', b'l', b'o', 3, 3, 3]);
    /// ```
    pub fn pkcs7_pad_in_place<const BLOCK_SIZE: usize, T: Copy>(data: &mut Vec<T>) {
        let data_len = data.len();
        let pad_num = unsafe { convert_usize_to::<T>(BLOCK_SIZE - data_len % BLOCK_SIZE) };
        data.resize((data.len() / BLOCK_SIZE + 1) * BLOCK_SIZE, pad_num);
    }
    /// PKCS#7 填充方案非标准实现
    ///
    /// ## PKCS#7 填充规则（非标准扩展版）
    /// - 填充单元大小小于一个 `usize`.
    /// - 如果数据长度不是块大小(BLOCK_SIZE)的整数倍，填充 `n` 个单元，每个单元值为 `n`
    ///   （`n` 是缺少的单元数）。
    /// - 如果数据长度恰好是块大小的整数倍，则填充一整块单元，每个单元值为块大小。
    ///
    /// ## 泛型参数要求
    /// - `BLOCK_SIZE` 为块大小，必须在 1-(2.pow(size_of::<T>())-1) 范围内。
    /// - `T` 为填充单元数据类型。标准 PKCS#7 要求为 u8 等单字节的数据类型，
    ///   本函数不作要求。但需保证：对于任意填充合法且允许通过字节直接初始化。
    ///
    /// ## 函数签名
    /// - `data` 为待填充数据。仅读取。
    /// - 返回 `Vec<[T; BLOCK_SIZE]>`, 为填充并分块后的数据。
    ///
    /// ## 示例
    /// ```
    /// // 8字节块大小
    /// use std::cell::UnsafeCell;
    /// let data = b"hello";
    /// let padded = cx_enc_utils::crypto::pkcs7_pad::<8, u8>(data);
    /// println!("{padded:?}");
    /// assert_eq!(padded[0], [b'h', b'e', b'l', b'l', b'o', 3, 3, 3]);
    /// ```
    pub fn pkcs7_pad<const BLOCK_SIZE: usize, T: Copy>(data: &[T]) -> Vec<[T; BLOCK_SIZE]> {
        let mut r = vec![[MaybeUninit::<T>::uninit(); BLOCK_SIZE]; data.len() / BLOCK_SIZE + 1];
        if size_of::<T>() != 0 {
            let buffer = r.as_mut_ptr().cast::<[T; BLOCK_SIZE]>();
            unsafe {
                pkcs7_pad_const(data, buffer);
            }
        }
        unsafe { mem::transmute(r) }
    }
    /// # Safety
    ///
    /// 调用此函数时，调用者**必须**遵守以下安全要求：
    ///
    /// ## 缓冲区容量验证
    /// buffer 指向的内存区域必须有**足够容量**容纳完整填充后的数据：
    /// 缓冲区容量 ≥ ceil(data.len() / BLOCK_SIZE) × BLOCK_SIZE 个元素。
    /// ## 指针有效性
    /// buffer 必须是**有效指针**，指向已分配且生命周期涵盖函数调用的内存区域。
    /// 内存必须**对齐**到类型 T 的对齐要求。
    /// 不允许 buffer 在调用期间被其他线程或上下文修改（无别名访问）。
    ///
    /// ## 类型 T 的约束
    /// - T 应为**整数类型**（或任意填充合法）。
    /// - 类型 T 必须允许通过字节直接初始化（无析构副作用）。
    ///
    /// ## 常量上下文限制
    /// 在 const 上下文中调用时，指针 buffer 需指向**编译期已知地址**（如 static 内存）。
    ///
    ///## 多线程安全
    /// 函数**不提供同步机制**，调用者须确保：
    /// - 无其他线程并发读写 buffer 指向的内存。
    /// - 对同一内存的独占访问。
    ///
    ///## 跨平台一致性
    ///   端序处理仅在支持 `#[cfg(target_endian)]` 的平台生效。
    ///  在未明确大小端序的**异构平台**（如某些嵌入式系统）上行为未定义。
    pub const unsafe fn pkcs7_pad_const<const BLOCK_SIZE: usize, T: Copy>(
        data: &[T],
        buffer: *mut [T; BLOCK_SIZE],
    ) {
        if size_of::<T>() == 0 {
            return;
        }
        let data_len = data.len();
        unsafe {
            ptr::copy_nonoverlapping(data.as_ptr(), buffer.cast::<T>(), data_len);
            pkcs7_pad_const_in_place(data_len, buffer);
        }
    }
    /// # Safety
    ///
    /// 调用此函数时，调用者**必须**遵守以下安全要求：
    ///
    /// ## 缓冲区容量验证
    /// buffer 指向的内存区域必须有**足够容量**容纳完整填充后的数据：
    /// 缓冲区容量 ≥ ceil(data.len() / BLOCK_SIZE) × BLOCK_SIZE 个元素。
    /// ## 指针有效性
    /// buffer 必须是**有效指针**，指向已分配且生命周期涵盖函数调用的内存区域。
    /// 内存必须**对齐**到类型 T 的对齐要求。
    /// 不允许 buffer 在调用期间被其他线程或上下文修改（无别名访问）。
    ///
    /// ## 类型 T 的约束
    /// - T 应为**整数类型**（或任意填充合法）。
    /// - 类型 T 必须允许通过字节直接初始化（无析构副作用）。
    ///
    /// ## 常量上下文限制
    /// 在 const 上下文中调用时，指针 buffer 需指向**编译期已知地址**（如 static 内存）。
    ///
    ///## 多线程安全
    /// 函数**不提供同步机制**，调用者须确保：
    /// - 无其他线程并发读写 buffer 指向的内存。
    /// - 对同一内存的独占访问。
    ///
    ///## 跨平台一致性
    ///   端序处理仅在支持 `#[cfg(target_endian)]` 的平台生效。
    ///  在未明确大小端序的**异构平台**（如某些嵌入式系统）上行为未定义。
    pub const unsafe fn pkcs7_pad_const_in_place<const BLOCK_SIZE: usize, T: Copy>(
        data_len: usize,
        buffer: *mut [T; BLOCK_SIZE],
    ) {
        if size_of::<T>() == 0 {
            return;
        }
        let pad_len = BLOCK_SIZE - data_len % BLOCK_SIZE;
        let pad_num = unsafe { convert_usize_to::<T>(BLOCK_SIZE - data_len % BLOCK_SIZE) };
        unsafe {
            let mut offset = data_len;
            while offset < data_len + pad_len {
                ptr::write(buffer.cast::<T>().add(offset), pad_num);
                offset += 1;
            }
        }
    }
    // TODO: 需要测试。
    const unsafe fn convert_usize_to<T: Copy>(data: usize) -> T {
        if size_of::<T>() == 0 {
            let t = MaybeUninit::<T>::uninit();
            return unsafe { t.assume_init() };
        }
        unsafe {
            struct Offset<T>(PhantomData<T>);
            impl<T> Offset<T> {
                const fn offset() -> usize {
                    #[cfg(target_endian = "big")]
                    {
                        size_of::<usize>() - size_of::<T>()
                    }
                    #[cfg(target_endian = "little")]
                    {
                        0
                    }
                }
            }
            let data = (&data) as *const usize as *const u8;
            *(data.add(Offset::<T>::offset()) as *const T)
        }
    }
    mod tests {
        #[test]
        fn test_convert_usize_to() {
            use crate::crypto::convert_usize_to;
            let data = 0xdeadbeef_baadc0de;
            println!("{:x}", unsafe { convert_usize_to::<u32>(data) })
        }
        #[test]
        fn test_pkcs7_pad_const() {
            const PADDED: [[u32; 8]; 2] = {
                use crate::crypto::pkcs7_pad_const;
                let mut padded = [[0; 8]; 2];
                unsafe {
                    pkcs7_pad_const::<8, u32>(
                        &[32, 32, 54, 56, 87, 36, 92, 12],
                        padded.as_mut_ptr().cast::<[u32; 8]>(),
                    );
                }
                padded
            };
            println!("{PADDED:?}");
            assert_eq!(PADDED[1], [8, 8, 8, 8, 8, 8, 8, 8]);
        }
    }
}
