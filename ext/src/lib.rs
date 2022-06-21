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
#[macro_use]
extern crate log;

#[cfg(feature = "serde")]
extern crate serde_crate as serde;

mod messages;

use internet2::{CreateUnmarshaller, Unmarshaller};
pub use messages::ExtMsg;
use once_cell::sync::Lazy;

pub const STORM_NODE_EXT_ENDPOINT: &str = "{data_dir}/storm";

pub static STORM_EXT_UNMARSHALLER: Lazy<Unmarshaller<ExtMsg>> =
    Lazy::new(|| ExtMsg::create_unmarshaller());
