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
