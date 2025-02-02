use crate::MaybeFatalError;
use onceinit::OnceInitError;
use thiserror::Error;
use ureq::Error;

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
        // Error::Status(_code, _r) => {
        //     //TODO
        //     true
        // }
        // Error::Transport(t) => {
        //     match t.kind() {
        //         // 说明可能是程序 Bug, 故视为致命错误。
        //         ErrorKind::InvalidUrl => true,
        //         // 说明可能是程序 Bug, 故视为致命错误。
        //         ErrorKind::UnknownScheme => true,
        //         //　有时会暂时性地解析失败。所以不视为致命错误。
        //         ErrorKind::Dns => false,
        //         ErrorKind::InsecureRequestHttpsOnly => true,
        //         ErrorKind::ConnectionFailed => true,
        //         ErrorKind::TooManyRedirects => false,
        //         ErrorKind::BadStatus => true,
        //         // 说明可能是程序 Bug, 故视为致命错误。
        //         ErrorKind::BadHeader => true,
        //         ErrorKind::Io => false,
        //         ErrorKind::InvalidProxyUrl => true,
        //         ErrorKind::ProxyConnect => true,
        //         ErrorKind::ProxyUnauthorized => true,
        //         ErrorKind::HTTP => {
        //             //TODO
        //             false
        //         }
        //     }
        // }
        use ureq_proto::Error as ProtoError;
        match &*self.0 {
            Error::StatusCode(code) => *code != 504,
            Error::Http(_) => true,
            Error::BadUri(_) => true,
            Error::Protocol(e) => !matches!(
                e,
                ProtoError::UnfinishedRequest | ProtoError::IncompleteResponse
            ),
            Error::Io(_) => false,
            Error::Timeout(_) => {
                // TODO
                true
            }
            Error::HostNotFound => true,
            Error::RedirectFailed => {
                // TODO
                false
            }
            Error::InvalidProxyUrl => true,
            Error::ConnectionFailed => true,
            Error::BodyExceedsLimit(_) => true,
            Error::TooManyRedirects => false,
            Error::RequireHttpsOnly(_) => false,
            Error::LargeResponseHeader(_, _) => false,
            Error::ConnectProxyFailed(_) => true,
            Error::BodyStalled => true,
            _ => {
                // TODO
                true
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
