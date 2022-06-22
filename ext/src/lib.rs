// Storm node providing distributed storage & messaging for lightning network.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

#![recursion_limit = "256"]

#[macro_use]
extern crate amplify;
#[macro_use]
extern crate strict_encoding;
#[macro_use]
extern crate internet2;

#[cfg(feature = "serde")]
extern crate serde_crate as serde;

mod messages;

use internet2::{CreateUnmarshaller, Unmarshaller};
pub use messages::ExtMsg;
use once_cell::sync::Lazy;

#[cfg(any(target_os = "linux"))]
pub const STORM_NODE_DATA_DIR: &str = "~/.storm_node";
#[cfg(any(target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
pub const STORM_NODE_DATA_DIR: &str = "~/.storm_node";
#[cfg(target_os = "macos")]
pub const STORM_NODE_DATA_DIR: &str = "~/Library/Application Support/Storm Node";
#[cfg(target_os = "windows")]
pub const STORM_NODE_DATA_DIR: &str = "~\\AppData\\Local\\Storm Node";
#[cfg(target_os = "ios")]
pub const STORM_NODE_DATA_DIR: &str = "~/Documents";
#[cfg(target_os = "android")]
pub const STORM_NODE_DATA_DIR: &str = ".";

pub const STORM_NODE_EXT_ENDPOINT: &str = const_format::formatcp!("{}/storm", STORM_NODE_DATA_DIR);

pub static STORM_EXT_UNMARSHALLER: Lazy<Unmarshaller<ExtMsg>> =
    Lazy::new(|| ExtMsg::create_unmarshaller());
