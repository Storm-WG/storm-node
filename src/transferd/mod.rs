// Storm node providing distributed storage & messaging for lightning network.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

mod service;
#[cfg(feature = "server")]
mod opts;
mod automation;

pub(self) use automation::State;
pub use automation::{AutomationError, Instruction, StateName};
#[cfg(feature = "server")]
pub use opts::Opts;
pub use service::{run, Runtime};
