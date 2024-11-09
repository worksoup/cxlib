mod default_impl;

pub use default_impl::*;

use onceinit::{OnceInit, OnceInitError, OnceInitState, StaticDefault};
use std::ops::Deref;
pub trait ProtocolTrait<Protocol>: Sync {
    fn get(&self, t: &Protocol) -> String;

    fn set(&self, t: &Protocol, value: &str);
    fn store(&self) -> Result<(), cxsign_error::Error>;
    fn update(&self, t: &Protocol, value: &str) -> bool;
}

static PROTOCOL: OnceInit<dyn ProtocolTrait<ProtocolItem>> = OnceInit::new();

impl StaticDefault for dyn ProtocolTrait<ProtocolItem> {
    fn static_default() -> &'static Self {
        if let OnceInitState::UNINITIALIZED = PROTOCOL.get_state() {
            let _ = CXProtocol::<ProtocolData>::init();
        }
        PROTOCOL.deref()
    }
}

pub fn set_protocol(
    protocol: &'static impl ProtocolTrait<ProtocolItem>,
) -> Result<(), OnceInitError> {
    PROTOCOL.set_data(protocol)
}

pub fn set_boxed_protocol(
    protocol: Box<impl ProtocolTrait<ProtocolItem> + 'static>,
) -> Result<(), OnceInitError> {
    PROTOCOL.set_boxed_data(protocol)
}
