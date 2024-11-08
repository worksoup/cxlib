#![feature(let_chains)]

pub mod cx_protocol;
pub mod protocol;
pub mod sign;
pub mod signner;
pub mod store;
pub mod utils;

/// 初始化所有需要初始化的默认实现。
pub fn init_all() -> Result<(), cxsign_error::Error> {
    cx_protocol::DefaultCXProtocol::init()
}
