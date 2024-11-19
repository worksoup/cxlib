use des::{
    cipher::{generic_array::GenericArray, BlockEncrypt as _, KeyInit as _},
    Des,
};
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
pub fn des_enc(data: &[u8], key: [u8; 8]) -> String {
    let key = GenericArray::from(key);
    let des = Des::new(&key);
    let mut data_block_enc = Vec::new();
    for block in pkcs7_pad(data) {
        let mut block = GenericArray::from(block);
        des.encrypt_block(&mut block);
        let mut block = block.to_vec();
        data_block_enc.append(&mut block);
    }
    hex::encode(data_block_enc)
}
