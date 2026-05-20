//! All structure needed by RustHound-CE.
//!
//! Example in rust:
//!
//! ```rust
//! # use rusthound_ce::objects::user::User;
//! # use rusthound_ce::objects::group::Group;
//! # use rusthound_ce::objects::computer::Computer;
//! # use rusthound_ce::objects::ou::Ou;
//! # use rusthound_ce::objects::gpo::Gpo;
//! # use rusthound_ce::objects::domain::Domain;
//! # use rusthound_ce::objects::container::Container;
//!
//! let user = User::new();
//! let group = Group::new();
//! let computer = Computer::new();
//! let ou = Ou::new();
//! let gpo = Gpo::new();
//! let domain = Domain::new();
//! let container = Container::new();
//! ```
//!

pub(crate) mod attribute;
pub(crate) mod directory_objects;
