use base64::{DecodeError, Engine};
use crypto::{
    aes, blockmodes,
    buffer::{BufferResult, ReadBuffer, RefReadBuffer, RefWriteBuffer, WriteBuffer},
};
use percent_encoding::PercentEncode;

fn aes_cbc_enc(padded_data: &[u8], key: &[u8; 16], iv: &[u8]) -> Vec<u8> {
    let mut encryptor =
        aes::cbc_encryptor(aes::KeySize::KeySize128, key, iv, blockmodes::NoPadding);
    let mut final_result = Vec::<u8>::new();
    let mut read_buffer = RefReadBuffer::new(padded_data);
    let mut buffer = [0; 4096];
    let mut write_buffer = RefWriteBuffer::new(&mut buffer);

    loop {
        let result = encryptor
            .encrypt(&mut read_buffer, &mut write_buffer, true)
            .expect("Encrypt failed");

        final_result.extend(
            write_buffer
                .take_read_buffer()
                .take_remaining()
                .iter()
                .copied(),
        );

        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => {}
        }
    }
    final_result
}
fn aes_cbc_dec(
    enc_data: &[u8],
    key: &[u8; 16],
    iv: &[u8],
) -> Result<Vec<u8>, crypto::symmetriccipher::SymmetricCipherError> {
    let mut decryptor =
        aes::cbc_decryptor(aes::KeySize::KeySize128, key, iv, blockmodes::PkcsPadding);

    let mut final_result = Vec::<u8>::new();
    let mut read_buffer = RefReadBuffer::new(enc_data);
    let mut buffer = [0; 4096];
    let mut write_buffer = RefWriteBuffer::new(&mut buffer);

    loop {
        let result = decryptor.decrypt(&mut read_buffer, &mut write_buffer, true)?;
        final_result.extend(
            write_buffer
                .take_read_buffer()
                .take_remaining()
                .iter()
                .copied(),
        );
        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => {}
        }
    }

    Ok(final_result)
}
fn aes_ecb_enc(padded_data: &[u8], key: &[u8; 16]) -> Vec<u8> {
    let mut encryptor = aes::ecb_encryptor(aes::KeySize::KeySize128, key, blockmodes::NoPadding);
    let mut final_result = Vec::<u8>::new();
    let mut read_buffer = RefReadBuffer::new(padded_data);
    let mut buffer = [0; 4096];
    let mut write_buffer = RefWriteBuffer::new(&mut buffer);

    loop {
        let result = encryptor
            .encrypt(&mut read_buffer, &mut write_buffer, true)
            .expect("Encrypt failed");

        final_result.extend(
            write_buffer
                .take_read_buffer()
                .take_remaining()
                .iter()
                .copied(),
        );

        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => {}
        }
    }
    final_result
}
fn aes_ecb_dec(
    enc_data: &[u8],
    key: &[u8; 16],
) -> Result<Vec<u8>, crypto::symmetriccipher::SymmetricCipherError> {
    let mut decryptor = aes::ecb_decryptor(aes::KeySize::KeySize128, key, blockmodes::PkcsPadding);

    let mut final_result = Vec::<u8>::new();
    let mut read_buffer = RefReadBuffer::new(enc_data);
    let mut buffer = [0; 4096];
    let mut write_buffer = RefWriteBuffer::new(&mut buffer);

    loop {
        let result = decryptor.decrypt(&mut read_buffer, &mut write_buffer, true)?;
        final_result.extend(
            write_buffer
                .take_read_buffer()
                .take_remaining()
                .iter()
                .copied(),
        );
        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => {}
        }
    }

    Ok(final_result)
}

#[inline]
fn percent_enc(input: &str) -> PercentEncode {
    percent_encoding::utf8_percent_encode(input, percent_encoding::NON_ALPHANUMERIC)
}
#[inline]
fn base64_enc<T: AsRef<[u8]>>(input: T) -> String {
    base64::engine::general_purpose::STANDARD.encode(input)
}
#[inline]
fn base64_dec<T: AsRef<[u8]>>(input: T) -> Result<Vec<u8>, DecodeError> {
    base64::engine::general_purpose::STANDARD.decode(input)
}
#[inline]
fn md5_enc<T: AsRef<[u8]>>(input: T) -> [u8; 16] {
    md5::compute(input).0
}
#[inline]
fn flatten_bytes<const BLOCK_SIZE: usize>(mut blocks: Vec<[u8; BLOCK_SIZE]>) -> Vec<u8> {
    let (p, l, c) = (blocks.as_mut_ptr(), blocks.len(), blocks.capacity());
    unsafe { Vec::from_raw_parts(p as *mut u8, l * BLOCK_SIZE, c * BLOCK_SIZE) }
}
#[inline]
// TODO: 传入 uid 时结果与预期不符，可能是有更新，需要逆向。
pub fn chaoxing_get_identifier(seed: impl AsRef<[u8]>) -> String {
    hex::encode(md5_enc(seed.as_ref()))
}
#[inline]
pub fn chaoxing_get_devicecode(ident: impl AsRef<[u8]>) -> Vec<u8> {
    let ident: Vec<[u8; 16]> = cx_enc_utils::crypto::pkcs7_pad(ident.as_ref());
    aes_ecb_enc(&flatten_bytes(ident), b"QrCbNY@MuK1X8HGw")
}
#[inline]
pub fn chaoxing_get_schild(part: impl std::fmt::Display) -> String {
    hex::encode(md5_enc(
        format!("(schild:ipL$TkeiEmfy1gTXb2XHrdLN0a@7c^vu) {part}").as_bytes(),
    ))
}

pub fn user_agent_gen(
    base_ua: &str,
    custom_ident: &str,
    schild: Option<&str>,
    device: &str,
    device_identifier: &str,
) -> String {
    let ua_part = format!("(device:{device}) {custom_ident} (@Kalimdor)_{device_identifier}",);
    if let Some(schild) = schild {
        return format!("{base_ua} (schild:{schild}) {ua_part}",);
    }
    let schild = chaoxing_get_schild(&ua_part);
    format!("{base_ua} (schild:{schild}) {ua_part}",)
}
pub fn user_agent_parse(
    ua: &str,
) -> (&str, Option<&str>, Option<&str>, Option<&str>, Option<&str>) {
    let pat = ["(schild:", "(device:", ")", "(@Kalimdor)_"];
    let mut result = [None; 5];
    let mut rest = ua;
    let mut res_index = 0;
    for (pat_index, pat) in pat.into_iter().enumerate() {
        if let Some(end) = rest.find(pat) {
            result[res_index] = Some(&rest[..end]);
            rest = &rest[end + pat.len()..];
            res_index = pat_index + 1;
        }
    }
    result[res_index] = Some(rest);
    assert!(result[0].is_some());
    fn f(s: &str) -> &str {
        let end = s.find(')');
        if let Some(end) = end { &s[..end] } else { s }.trim()
    }
    (
        result[0].unwrap().trim(),
        result[1].map(f),
        result[2].map(str::trim),
        result[3].map(str::trim),
        result[4].map(str::trim),
    )
}

#[cfg(test)]
mod tests {

    use crate::{chaoxing_get_identifier, chaoxing_get_schild, user_agent_gen, user_agent_parse};

    #[test]
    fn tmp() {
        let uid = "317347528";
        let a = chaoxing_get_identifier(uid);
        println!("{a}");
        let b = chaoxing_get_schild(
            "(device:V2118A) Language/zh_CN com.chaoxing.mobile/ChaoXingStudy_3_6.6.0_android_phone_10893_283 (@Kalimdor)_eb21ec75ce62463ea067895e3340f627",
        );
        println!("{b}");
    }
    #[test]
    fn gen_and_parse() {
        use std::time::Instant;
        let uid = "317347528";
        let identifier = chaoxing_get_identifier(uid);
        let start_time = Instant::now();
        let ua = user_agent_gen(
            "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0.0.0 Mobile Safari/537.36",
            "Language/zh_CN com.chaoxing.mobile/ChaoXingStudy_3_6.6.0_android_phone_10893_283",
            None,
            "V2118A",
            &identifier,
        );
        let elapsed = start_time.elapsed();
        println!("Generated User Agent: {ua}");
        println!("Generated in: {elapsed:?}");
        let start_time = Instant::now();
        let (base_ua, schild, device, custom_ident, device_identifier) = user_agent_parse(&ua);
        let elapsed = start_time.elapsed();
        println!("Parsed in: {elapsed:?}");
        println!("Base UA: {base_ua}");
        println!("Schild: {schild:?}");
        println!("Device: {device:?}");
        println!("Custom Ident: {custom_ident:?}");
        println!("Device Identifier: {device_identifier:?}");
    }
}
