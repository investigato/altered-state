pub(crate) mod config;
pub mod ldap;
pub(crate) mod objects;
pub(crate) mod models;
pub(crate) mod storage;
pub(crate) mod utilities;
extern crate bitflags;
extern crate chrono;
extern crate regex;

// Reimport key functions and structure
#[doc(inline)]
pub use ldap::ldap_search;
#[doc(inline)]
pub use ldap3::SearchEntry;

pub use ldap::prepare_results_from_source;
// pub use json::maker::make_result;
pub use objects::{
    attribute::SchemaEntry, directory_objects::save_directory_objects_to_bin_file,
};
pub use storage::{DiskStorage, DiskStorageReader, EntrySource, Storage};
pub use utilities::banner::print_banner;
