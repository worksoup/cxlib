pub fn url_encode(input: &str) -> String {
    percent_encoding::utf8_percent_encode(input, percent_encoding::NON_ALPHANUMERIC).to_string()
}
pub use zlib_impl::*;
mod zlib_impl {
    use std::io::Read;

    pub fn zlib_encode(text: &str) -> Vec<u8> {
        use flate2::write::ZlibEncoder;
        use flate2::Compression;
        use std::io::prelude::*;
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(text.as_bytes()).unwrap();
        encoder.finish().unwrap()
    }

    pub fn zlib_decode<R: Read>(r: R) -> String {
        let mut decoder = ZlibDecoder::new(r);
        use flate2::read::ZlibDecoder;
        let mut decompressed_data = String::new();
        decoder.read_to_string(&mut decompressed_data).unwrap();
        decompressed_data
    }
}
