use cxlib_error_utils::MaybeFatalError;
use thiserror::Error;
use ureq::Error;

/// ureq 中 返回的错误。
///
/// 如果出现 504、HTTP 错误、Uri 错误、连接失败、代理地址无效、
/// 代理连接失败、Body 长度超限等短时间内无法通过重试解决的问题，
/// [`<AgentError as MaybeFatalError>::is_fatal`](MaybeFatalError::is_fatal) 将会返回 `true`,
/// 应当视为致命错误，代表在下一个循环或操作中依然会发生错误，此时应当结束操作，避免重复，浪费时间。
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
        // use ureq_proto::Error as ProtoError;
        match &*self.0 {
            Error::StatusCode(code) => *code != 504,
            Error::Http(_) => true,
            Error::BadUri(_) => true,
            Error::Protocol(_e) => {
                // TODO
                false
            }
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
