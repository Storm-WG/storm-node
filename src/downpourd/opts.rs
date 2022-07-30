// Storm node providing distributed storage & messaging for lightning network.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use clap::Parser;

use crate::opts::{Options, Opts as SharedOpts};

/// Command-line arguments
#[derive(Parser)]
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[clap(author, version, name = "downpourd", about = "Storm node file transfer service")]
pub struct Opts {
    /// These params can be read also from the configuration file, not just
    /// command-line args or environment variables
    #[clap(flatten)]
    pub shared: SharedOpts,
}

impl Options for Opts {
    type Conf = ();

    fn shared(&self) -> &crate::opts::Opts { &self.shared }

    fn config(&self) -> Self::Conf { () }
}

#[cfg(feature = "server")]
impl Opts {
    pub fn process(&mut self) { self.shared.process(); }
}
