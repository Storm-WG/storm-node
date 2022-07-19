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

/// Lightning storm daemon; part of Storm Node.
///
/// The daemon is controlled though RPC socket (see `rpc-endpoint`).
#[derive(Parser, Clone, PartialEq, Eq, Debug)]
#[clap(name = "stormd", bin_name = "stormd", author, version)]
pub struct Opts {
    /// These params can be read also from the configuration file, not just
    /// command-line args or environment variables
    #[clap(flatten)]
    pub shared: crate::opts::Opts,

    /// Run chat service.
    #[clap(long)]
    pub chat: bool,

    /// Run downpour (torrent-like) service.
    #[clap(long)]
    pub downpour: bool,

    /// Spawn daemons as threads and not processes
    #[clap(short = 'T', long = "threaded")]
    pub threaded_daemons: bool,
}

impl Opts {
    pub fn process(&mut self) { self.shared.process() }
}
