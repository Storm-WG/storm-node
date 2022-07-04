// Storm node providing distributed storage & messaging for lightning network.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

#[macro_use]
extern crate amplify;
#[macro_use]
extern crate internet2;
#[macro_use]
extern crate log;
#[macro_use]
extern crate strict_encoding;

mod config;
mod error;
pub mod stormd;
pub mod bus;
#[cfg(feature = "server")]
pub mod opts;

pub use config::Config;
pub(crate) use error::DaemonError;
pub use error::LaunchError;
