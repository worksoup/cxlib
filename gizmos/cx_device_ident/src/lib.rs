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

pub fn user_agent_gen(base_ua: &str, device_code: &str, device_identifier: &str) {}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use crate::{chaoxing_get_identifier, chaoxing_get_schild};

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
    fn md5_bench() {
        println!("MD5 计算性能测试开始...");

        // 测试参数
        let test_duration = Duration::from_secs(60); // 1分钟测试
        let data_size = 1024; // 每次计算的数据大小（字节）

        // 创建测试数据（随机内容）
        let test_data: Vec<u8> = (0..data_size).map(|i| (i % 256) as u8).collect();

        let start_time = Instant::now();
        let mut hash_count = 0;

        println!("测试运行中（将持续 60 秒）...");
        // 主测试循环
        while start_time.elapsed() < test_duration {
            // 计算 MD5 哈希
            let mut hasher = md5::Context::new();
            hasher.consume(&test_data);
            hasher.finalize(); // 故意不保存结果，只计算

            hash_count += 1;

            // 每秒显示一次进度
            if hash_count % 100_000 == 0 {
                let elapsed = start_time.elapsed().as_secs_f64();
                let hash_rate = hash_count as f64 / elapsed;
                println!(
                    "已计算: {hash_count} 次 | 当前速率: {hash_rate:.0} 次/秒"
                );
            }
        }

        let elapsed = start_time.elapsed().as_secs_f64();
        let hash_rate = hash_count as f64 / elapsed;

        println!("\n测试完成！");
        println!("总计算次数: {hash_count}");
        println!("总耗时: {elapsed:.2} 秒");
        println!("平均速率: {hash_rate:.0} 次/秒");
        println!("一分钟理论计算量: {:.0} 次", hash_rate * 60.0);
    }
}
