use cxsign_error::Error;
use log::warn;
use std::path::Path;
use ureq::{Agent, AgentBuilder};

pub mod protocol;
pub mod utils;
static UA: &str = "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36 com.chaoxing.mobile.xuezaixidian/ChaoXingStudy_1000149_5.3.1_android_phone_5000_83";
pub trait LoginTrait: Sized {
    fn login_enc(account: &str, enc_passwd: &str) -> Result<Self, Error>;

    fn load_json<P: AsRef<Path>>(cookies_file: P) -> Result<Self, std::io::Error>;
}
impl LoginTrait for Agent {
    fn login_enc(account: &str, enc_passwd: &str) -> Result<Agent, Error> {
        let cookie_store = cookie_store::CookieStore::new(None);
        let client = AgentBuilder::new()
            .user_agent(UA)
            .cookie_store(cookie_store)
            .build();
        let response = protocol::login_enc(&client, account, enc_passwd)?;
        /// TODO: 存疑
        #[derive(serde::Deserialize)]
        struct LoginR {
            url: Option<String>,
            msg1: Option<String>,
            msg2: Option<String>,
            status: bool,
        }
        let LoginR {
            status,
            url,
            msg1,
            msg2,
        } = response.into_json().unwrap();
        let mut mes = Vec::new();
        if let Some(url) = url {
            mes.push(url);
        }
        if let Some(msg1) = msg1 {
            mes.push(msg1);
        }
        if let Some(msg2) = msg2 {
            mes.push(msg2);
        }
        if !status {
            for mes in &mes {
                warn!("{mes:?}");
            }
            return Err(Error::LoginError(format!("{mes:?}")));
        }
        Ok(client)
    }

    fn load_json<P: AsRef<Path>>(cookies_file: P) -> Result<Agent, std::io::Error> {
        let cookie_store = {
            let file = std::fs::File::open(cookies_file).map(std::io::BufReader::new)?;
            cookie_store::CookieStore::load_json(file).unwrap()
        };
        Ok(AgentBuilder::new()
            .user_agent(UA)
            .cookie_store(cookie_store)
            .build())
    }
}

#[cfg(test)]
mod tests {}
