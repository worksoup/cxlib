use cx_enc_utils::{
    coding::{base64, hex},
    crypto::{Aes128, Mode},
};
#[inline]
// TODO: 传入 uid 时结果与预期不符，可能是有更新，需要逆向。
// 猜测可能为 cx_obfuscate?
pub fn chaoxing_get_identifier(seed: impl AsRef<[u8]>) -> String {
    hex::encode(cx_enc_utils::hash::md5_hash(seed.as_ref()))
}
// TODO: 传入 ident 时结果与预期不符，可能是有更新，需要逆向。
// 猜测可能为 SHA512?
#[inline]
pub fn chaoxing_get_devicecode(ident: impl AsRef<[u8]>) -> String {
    base64::encode(
        cx_enc_utils::crypto::Aes::<Aes128>::new(Mode::ECB, cx_enc_utils::crypto::Padding::Pkcs7)
            .enc(ident.as_ref(), b"QrCbNY@MuK1X8HGw".into()),
    )
}
#[inline]
pub fn chaoxing_get_schild(part: impl std::fmt::Display) -> String {
    hex::encode(cx_enc_utils::hash::md5_hash(
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

    use cx_enc_utils::{
        coding::base64,
        crypto::{Aes128, Mode, Padding},
    };

    use crate::{
        chaoxing_get_devicecode, chaoxing_get_identifier, chaoxing_get_schild, user_agent_gen,
        user_agent_parse,
    };

    #[test]
    fn tmp() {
        let uid = "317347528";
        let a = chaoxing_get_identifier(uid);
        println!("{a}");
        let b = chaoxing_get_schild(
            "(device:V2118A) Language/zh_CN com.chaoxing.mobile/ChaoXingStudy_3_6.6.0_android_phone_10893_283 (@Kalimdor)_eb21ec75ce62463ea067895e3340f627",
        );
        println!("{b}");
        assert_eq!(b, "45ea260fe4027811076d21570808258c");
    }
    #[test]
    fn gen_and_parse() {
        use std::time::Instant;
        let start_time = Instant::now();
        let ua = user_agent_gen(
            "Mozilla/5.0 (Linux; Android 14; V2118A Build/UP1A.231005.007; wv) AppleWebKit/537.36 (KHTML, like Gecko) Version/4.0 Chrome/138.0.7204.179 Mobile Safari/537.36",
            "Language/zh_CN com.chaoxing.mobile/ChaoXingStudy_3_6.6.0_android_phone_10893_283",
            None,
            "V2118A",
            "eb21ec75ce62463ea067895e3340f627",
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
        if let Some(device_identifier) = device_identifier {
            let device_code = chaoxing_get_devicecode(device_identifier);
            println!("{device_code}");
        }
        let device_code = "zE9+rwe3eRV6GaU0FX5KhiCnFj/Ju8X05ywjTUWxx+MV5Sfjflk+dHuRq2Pi3T7redETzSIYMbwqqRViHAEs88o+3W2FKfGichnoLiFd2iU=";
        let device_code = base64::decode(device_code).unwrap();
        let ident = cx_enc_utils::crypto::Aes::<Aes128>::new(Mode::ECB, Padding::Pkcs7)
            .dec(&device_code, b"QrCbNY@MuK1X8HGw".into());
        let ident = unsafe { String::from_utf8_unchecked(ident) };
        println!("{ident}");
        let device_code = base64::decode(ident).unwrap();
        println!("{device_code:?}",);
        let ident = unsafe { String::from_utf8_unchecked(device_code) };
        println!("{ident}");
    }
}
