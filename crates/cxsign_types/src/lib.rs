#![feature(let_chains)]
#![feature(map_try_insert)]
#![feature(sync_unsafe_cell)]
mod course;
mod dioption;
mod location;
mod photo;
pub mod protocol;
mod triple;

pub use course::*;
pub use dioption::*;
pub use location::*;
pub use photo::*;
pub use triple::*;
