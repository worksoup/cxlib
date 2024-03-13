use ureq::{Agent, Response};

// 获取课程
static BACK_CLAZZ_DATA: &str = "http://mooc1-api.chaoxing.com/mycourse/backclazzdata";

pub fn back_clazz_data(client: &Agent) -> Result<Response, ureq::Error> {
    client
        .get(&format!("{BACK_CLAZZ_DATA}?view=json&rss=1"))
        .call()
}
