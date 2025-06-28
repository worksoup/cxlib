pub mod collect;
mod default_impl;
mod error;
#[cfg(feature = "multipart")]
mod multipart;
#[cfg(feature = "ureq")]
pub mod utils;

pub use default_impl::*;
pub use error::*;

pub use cxlib_error::AgentError;

pub trait ProtocolItemTrait {
    type ProtocolData;
    fn get_default(&self) -> String;
}
pub trait ProtocolDataTrait {
    type ProtocolItem;
    fn config_file_name() -> &'static str;
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
