use crate::MaybeFatalError;
use onceinit::OnceInitError;
use thiserror::Error;
use ureq::{Error, ErrorKind};

#[derive(Error, Debug)]
#[error(transparent)]
pub struct AgentError(#[from] Box<ureq::Error>);
impl From<ureq::Error> for AgentError {
    fn from(value: ureq::Error) -> Self {
        Self(Box::new(value))
    }
}
impl MaybeFatalError for AgentError {
    fn is_fatal(&self) -> bool {
        match &*self.0 {
            Error::Status(_code, _r) => {
                //TODO
                true
            }
            Error::Transport(t) => {
                match t.kind() {
                    // 说明可能是程序 Bug, 故视为致命错误。
                    ErrorKind::InvalidUrl => true,
                    // 说明可能是程序 Bug, 故视为致命错误。
                    ErrorKind::UnknownScheme => true,
                    //　有时会暂时性地解析失败。所以不视为致命错误。
                    ErrorKind::Dns => false,
                    ErrorKind::InsecureRequestHttpsOnly => true,
                    ErrorKind::ConnectionFailed => true,
                    ErrorKind::TooManyRedirects => false,
                    ErrorKind::BadStatus => true,
                    // 说明可能是程序 Bug, 故视为致命错误。
                    ErrorKind::BadHeader => true,
                    ErrorKind::Io => false,
                    ErrorKind::InvalidProxyUrl => true,
                    ErrorKind::ProxyConnect => true,
                    ErrorKind::ProxyUnauthorized => true,
                    ErrorKind::HTTP => {
                        //TODO
                        false
                    }
                }
            }
        }
    }
}

#[derive(Error, Debug)]
#[error(transparent)]
pub struct InitError(#[from] OnceInitError);
impl MaybeFatalError for InitError {
    fn is_fatal(&self) -> bool {
        false
    }
}
