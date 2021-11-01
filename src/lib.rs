pub extern crate capabilities_derive;

pub use ::capabilities_derive::capability;
pub use ::capabilities_derive::service;

pub enum Operations {
    Create,
    Read,
    Update,
    Delete,
    CreateAll,
    ReadAll,
    UpdateAll,
    DeleteAll,
}
