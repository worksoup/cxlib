/// 区分签到类型时获取的一些签到的信息。
#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SignDetail {
    is_photo: bool,
    is_refresh_qrcode: bool,
    c: Option<String>,
}
impl SignDetail {
    #[inline]
    pub fn new(
        is_photo_value: i64,
        is_refresh_qrcode_value: i64,
        sign_code: Option<String>,
    ) -> SignDetail {
        SignDetail {
            is_photo: is_photo_value > 0,
            is_refresh_qrcode: is_refresh_qrcode_value > 0,
            c: sign_code,
        }
    }
    #[inline]
    pub fn is_photo(&self) -> bool {
        self.is_photo
    }
    #[inline]
    pub fn is_refresh_qrcode(&self) -> bool {
        self.is_refresh_qrcode
    }
    #[inline]
    pub fn sign_code(&self) -> Option<&str> {
        self.c.as_deref()
    }
}
