use proc_macro::TokenStream;
use quote::quote;

const _10: [u32; 64] = [
    0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
    0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
    0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
    0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed, 0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
    0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
    0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
    0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
    0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391,
];
const _11: [u32; 16] = [
    0x07, 0x0c, 0x11, 0x16, 0x05, 0x09, 0x0e, 0x14, 0x04, 0x0b, 0x10, 0x17, 0x06, 0x0a, 0x0f, 0x15,
];
const _12: [usize; 64] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
    0x01, 0x06, 0x0b, 0x00, 0x05, 0x0a, 0x0f, 0x04, 0x09, 0x0e, 0x03, 0x08, 0x0d, 0x02, 0x07, 0x0c,
    0x05, 0x08, 0x0b, 0x0e, 0x01, 0x04, 0x07, 0x0a, 0x0d, 0x00, 0x03, 0x06, 0x09, 0x0c, 0x0f, 0x02,
    0x00, 0x07, 0x0e, 0x05, 0x0c, 0x03, 0x0a, 0x01, 0x08, 0x0f, 0x06, 0x0d, 0x04, 0x0b, 0x02, 0x09,
];
static _00: [u8; 486] = [
    120, 156, 149, 148, 95, 143, 155, 48, 12, 192, 191, 74, 78, 211, 42, 103, 205, 16, 109, 233,
    95, 10, 210, 77, 154, 166, 189, 236, 225, 30, 38, 77, 136, 162, 16, 146, 54, 18, 164, 85, 9,
    59, 110, 109, 191, 251, 12, 92, 79, 247, 208, 158, 4, 66, 113, 20, 108, 255, 108, 99, 71, 25,
    178, 227, 229, 46, 129, 162, 178, 68, 155, 67, 101, 87, 191, 165, 88, 87, 147, 113, 200, 114,
    105, 86, 85, 169, 255, 73, 250, 53, 140, 240, 196, 247, 226, 211, 167, 72, 155, 92, 27, 9, 60,
    127, 230, 47, 37, 141, 149, 33, 137, 114, 19, 224, 43, 212, 96, 105, 179, 162, 58, 174, 39,
    238, 60, 31, 249, 225, 160, 205, 54, 225, 89, 6, 41, 189, 220, 49, 190, 97, 155, 75, 75, 68, 0,
    124, 224, 214, 10, 31, 58, 132, 244, 186, 245, 1, 128, 135, 225, 104, 214, 28, 190, 74, 209,
    74, 186, 94, 19, 20, 103, 16, 87, 221, 155, 196, 155, 192, 46, 7, 134, 65, 138, 189, 41, 45,
    249, 246, 253, 199, 207, 95, 201, 227, 211, 211, 227, 159, 213, 107, 238, 65, 228, 214, 179,
    185, 55, 29, 79, 220, 17, 115, 107, 169, 68, 198, 211, 197, 18, 183, 203, 69, 202, 51, 161, 36,
    110, 71, 238, 100, 60, 245, 230, 179, 216, 191, 137, 30, 65, 132, 16, 38, 216, 95, 158, 87,
    146, 149, 59, 173, 44, 43, 248, 86, 139, 184, 195, 204, 226, 119, 21, 72, 3, 140, 21, 223, 148,
    113, 202, 80, 118, 70, 173, 58, 165, 126, 243, 193, 57, 238, 45, 183, 50, 201, 165, 178, 208,
    122, 163, 76, 220, 73, 123, 12, 81, 67, 46, 152, 185, 75, 159, 95, 233, 77, 160, 88, 198, 2,
    139, 249, 32, 6, 134, 222, 137, 249, 14, 105, 210, 159, 100, 144, 84, 12, 30, 250, 146, 188,
    158, 36, 177, 41, 54, 166, 31, 98, 218, 19, 81, 108, 64, 156, 63, 74, 164, 249, 179, 117, 128,
    195, 245, 101, 225, 183, 19, 23, 213, 97, 56, 141, 207, 193, 104, 188, 88, 175, 161, 254, 140,
    77, 233, 55, 74, 173, 113, 194, 109, 112, 214, 221, 28, 158, 59, 138, 86, 68, 7, 129, 91, 187,
    114, 136, 179, 80, 15, 103, 30, 13, 195, 37, 118, 191, 71, 79, 53, 225, 37, 65, 165, 139, 204,
    75, 121, 106, 221, 59, 91, 105, 65, 83, 71, 236, 15, 90, 102, 64, 157, 202, 52, 131, 153, 236,
    143, 73, 38, 21, 175, 114, 11, 244, 114, 105, 137, 205, 37, 192, 131, 119, 189, 255, 118, 154,
    250, 106, 127, 36, 26, 175, 8, 2, 174, 227, 116, 142, 49, 7, 160, 212, 41, 173, 60, 36, 233,
    11, 52, 221, 79, 79, 105, 192, 253, 255, 92, 184, 103, 83,
];
static _01: [u8; 41] = [
    120, 156, 75, 203, 47, 82, 200, 84, 200, 204, 83, 48, 208, 211, 51, 169, 78, 140, 206, 140,
    181, 77, 51, 208, 72, 2, 210, 58, 32, 142, 166, 117, 109, 109, 98, 45, 0, 220, 44, 12, 13,
];
fn f0() -> String {
    let mut x = String::new();
    for i in 0..64 {
        let a = (4 - i % 4) % 4;
        let b = (5 - i % 4) % 4;
        let c = (6 - i % 4) % 4;
        let d = (7 - i % 4) % 4;
        let e = _12[i];
        let f = _11[i % 4 + ((i >> 4) << 2)];
        let g = _10[i];
        let h = match i / 16 {
            0 => "f2",
            1 => "f3",
            2 => "f4",
            3 => "f5",
            _ => {
                unreachable!()
            }
        };
        x.push_str(&format!(
            "a[{a}]={h}([a[{a}],a[{b}],a[{c}],a[{d}],value_at({}),{f},{g}]);",
            if e == 0 {
                "i".to_string()
            } else {
                "i+".to_string() + e.to_string().as_str()
            }
        ))
    }
    x
}
#[proc_macro]
pub fn __define(_input: TokenStream) -> TokenStream {
    use cxlib_utils::zlib_decode;
    let r = zlib_decode(&_00[..]) + &f0() + &zlib_decode(&_01[..]);
    let mut tokens = proc_macro2::TokenStream::new();
    let token: proc_macro2::TokenStream = r.parse().unwrap();
    tokens.extend(quote! {#token});
    tokens.into()
}
#[cfg(test)]
mod tests {
    static _10: [u8; 653] = [
        120, 156, 181, 148, 93, 107, 219, 48, 20, 134, 239, 251, 43, 212, 148, 5, 105, 85, 141,
        243, 217, 52, 142, 61, 58, 24, 99, 55, 187, 232, 197, 96, 24, 87, 200, 138, 156, 10, 108,
        37, 216, 242, 234, 214, 241, 127, 223, 145, 211, 132, 176, 197, 163, 185, 152, 8, 214, 215,
        123, 244, 232, 156, 232, 156, 11, 4, 173, 44, 36, 18, 57, 55, 114, 62, 175, 19, 151, 34,
        230, 182, 159, 65, 227, 93, 28, 182, 171, 66, 173, 52, 43, 141, 74, 11, 80, 189, 166, 42,
        102, 75, 41, 214, 75, 73, 81, 59, 145, 218, 78, 192, 162, 53, 185, 10, 141, 44, 76, 212,
        142, 19, 141, 236, 132, 49, 76, 80, 221, 174, 216, 150, 74, 131, 114, 228, 163, 163, 147,
        112, 31, 192, 161, 227, 68, 196, 59, 200, 54, 185, 210, 38, 213, 151, 184, 87, 231, 77,
        239, 228, 198, 77, 87, 59, 150, 239, 113, 137, 139, 255, 227, 233, 127, 56, 51, 120, 143,
        51, 205, 169, 128, 49, 102, 167, 127, 7, 204, 63, 138, 53, 238, 129, 240, 137, 23, 79, 12,
        103, 165, 65, 74, 111, 74, 51, 255, 33, 197, 162, 28, 13, 3, 154, 74, 61, 47, 11, 245, 42,
        201, 77, 16, 194, 138, 55, 142, 234, 171, 80, 233, 84, 105, 137, 121, 250, 204, 95, 10, 18,
        89, 82, 226, 50, 204, 231, 160, 160, 177, 253, 130, 28, 190, 53, 119, 158, 115, 190, 217,
        40, 189, 98, 124, 185, 196, 49, 105, 58, 140, 79, 216, 218, 155, 10, 31, 243, 190, 91, 37,
        208, 200, 53, 142, 247, 67, 15, 99, 204, 131, 96, 48, 181, 139, 111, 189, 104, 123, 178,
        88, 32, 232, 182, 88, 236, 181, 39, 137, 39, 129, 59, 31, 40, 92, 82, 172, 117, 97, 208,
        231, 47, 95, 191, 125, 103, 247, 15, 15, 247, 63, 231, 111, 190, 251, 161, 91, 77, 111,
        199, 147, 225, 200, 29, 80, 183, 146, 137, 88, 242, 120, 118, 7, 195, 187, 89, 204, 151,
        34, 145, 48, 28, 184, 163, 225, 100, 124, 59, 141, 188, 147, 232, 1, 14, 1, 66, 5, 253,
        197, 211, 82, 210, 226, 73, 37, 134, 102, 124, 165, 68, 180, 195, 76, 163, 163, 8, 196, 62,
        220, 21, 126, 49, 229, 132, 66, 191, 51, 106, 229, 132, 120, 118, 195, 201, 215, 6, 146,
        142, 165, 50, 49, 184, 61, 141, 80, 209, 225, 246, 16, 135, 150, 156, 81, 221, 73, 191,
        221, 211, 237, 69, 33, 140, 25, 4, 243, 82, 244, 53, 233, 184, 115, 7, 105, 116, 62, 73, 3,
        41, 235, 95, 158, 75, 26, 159, 73, 18, 143, 217, 163, 62, 15, 49, 57, 19, 145, 61, 98, 177,
        253, 151, 35, 246, 159, 173, 124, 72, 174, 143, 51, 175, 205, 184, 176, 10, 130, 73, 180,
        245, 7, 195, 217, 98, 129, 171, 15, 240, 40, 61, 43, 106, 141, 25, 55, 254, 86, 237, 242,
        112, 187, 163, 168, 4, 41, 223, 119, 43, 87, 94, 67, 46, 84, 215, 211, 49, 9, 130, 59, 120,
        253, 99, 82, 87, 136, 23, 8, 68, 141, 76, 11, 89, 183, 199, 59, 43, 105, 176, 34, 142, 88,
        111, 148, 92, 98, 226, 148, 218, 38, 38, 91, 231, 80, 103, 18, 94, 166, 80, 36, 154, 166,
        37, 218, 34, 192, 253, 163, 183, 127, 88, 141, 189, 100, 157, 35, 5, 37, 2, 97, 215, 113,
        118, 7, 131, 15, 152, 16, 167, 48, 114, 195, 226, 23, 108, 95, 63, 169, 99, 159, 123, 189,
        142, 138, 53, 255, 212, 116, 214, 188, 67, 69, 218, 115, 0, 51, 174, 121, 168, 34, 155, 5,
        49, 244, 212, 78, 136, 215, 52, 188, 121, 7, 160, 57, 8, 108, 251, 13, 30, 117, 255, 144,
    ];
}