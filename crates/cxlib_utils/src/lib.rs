pub use crypto::*;
pub use debug::*;
pub use interact::*;
pub use ureq::*;
mod debug;
mod interact;
mod ureq {
    pub fn ureq_get_bytes(
        agent: &ureq::Agent,
        url: &str,
        referer: &str,
    ) -> Result<Vec<u8>, Box<ureq::Error>> {
        let mut img = Vec::new();
        agent
            .get(url)
            .set("Referer", referer)
            .call()?
            .into_reader()
            .read_to_end(&mut img)
            .unwrap();
        Ok(img)
    }
}

mod crypto {
    pub fn pkcs7_pad<const BLOCK_SIZE: usize>(data: &[u8]) -> Vec<[u8; BLOCK_SIZE]> {
        let len = data.len();
        let batch = len / BLOCK_SIZE;
        let m = len % BLOCK_SIZE;
        let len2 = BLOCK_SIZE - m;
        let mut r = vec![[0u8; BLOCK_SIZE]; batch + 1];
        let pad_num = ((BLOCK_SIZE - m) % 0xFF) as u8;
        let r_data = r.as_mut_ptr() as *mut u8;
        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), r_data, len);
            std::ptr::copy_nonoverlapping(
                vec![pad_num; len2].as_ptr(),
                r_data.add(batch * BLOCK_SIZE + m),
                len2,
            );
        }
        r
    }
}
