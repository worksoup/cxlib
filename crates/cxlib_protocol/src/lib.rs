pub mod collect;
mod default_impl;
#[cfg(feature = "multipart")]
mod multipart;
#[cfg(feature = "ureq")]
pub mod utils;

pub use default_impl::*;

use cxlib_error::InitError;
use onceinit::{OnceInit, OnceInitState, StaticDefault};

pub use cxlib_error::ProtocolError;

pub trait ProtocolItemTrait: Sized + 'static {
    type ProtocolData;
    fn config_file_name() -> &'static str;
    fn get_protocol_() -> &'static OnceInit<dyn ProtocolTrait<Self>>;
    fn get_protocol() -> &'static dyn ProtocolTrait<Self>;
    fn set_protocol(protocol: &'static impl ProtocolTrait<Self>) -> Result<(), InitError> {
        Ok(Self::get_protocol_().set_data(protocol)?)
    }
    fn set_boxed_protocol(
        protocol: Box<impl ProtocolTrait<Self> + 'static>,
    ) -> Result<(), InitError> {
        Ok(Self::get_protocol_().set_boxed_data(protocol)?)
    }
    fn get(&self) -> String {
        Self::get_protocol().get(self)
    }
    fn get_default(&self) -> String;
    fn set(&self, value: &str) {
        Self::get_protocol().set(self, value)
    }
    fn store() -> Result<(), ProtocolError> {
        Self::get_protocol().store()
    }
    fn update(&self, value: &str) -> bool {
        Self::get_protocol().update(self, value)
    }
}
pub trait ProtocolDataTrait {
    type ProtocolItem;
    fn map_by_enum<'a, T>(
        &'a self,
        t: &Self::ProtocolItem,
        do_something: impl Fn(&'a Option<String>) -> T,
    ) -> T;
    fn map_by_enum_mut<'a, T>(
        &'a mut self,
        t: &Self::ProtocolItem,
        do_something: impl Fn(&'a mut Option<String>) -> T,
    ) -> T;
    fn set(&mut self, t: &Self::ProtocolItem, value: &str) {
        self.map_by_enum_mut(t, |t| t.replace(value.to_owned()));
    }
    fn update(&mut self, t: &Self::ProtocolItem, value: &str) -> bool {
        self.map_by_enum_mut(t, |t| {
            let not_to_update = t.as_ref().is_some_and(|v| v == value);
            t.replace(value.to_owned());
            !not_to_update
        })
    }
}
pub trait ProtocolTrait<ProtocolItem>: Sync {
    fn get(&self, t: &ProtocolItem) -> String;

    fn set(&self, t: &ProtocolItem, value: &str);
    fn store(&self) -> Result<(), ProtocolError>;
    fn update(&self, t: &ProtocolItem, value: &str) -> bool;
}

static PROTOCOL: OnceInit<dyn ProtocolTrait<ProtocolItem>> = OnceInit::new();

unsafe impl<ProtocolItem, ProtocolData> StaticDefault for dyn ProtocolTrait<ProtocolItem>
where
    ProtocolItem: ProtocolItemTrait<ProtocolData = ProtocolData>,
    ProtocolData: Default
        + for<'de> serde::Deserialize<'de>
        + serde::Serialize
        + Send
        + Sync
        + 'static
        + ProtocolDataTrait<ProtocolItem = ProtocolItem>,
{
    fn static_default() -> &'static Self {
        if let OnceInitState::UNINITIALIZED = ProtocolItem::get_protocol_().get_state() {
            let _ = CXProtocol::<ProtocolData>::init();
        }
        ProtocolItem::get_protocol()
    }
}
