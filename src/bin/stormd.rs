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

//! Main executable for Storm node.

#[macro_use]
extern crate log;

use clap::Parser;
use microservices::error::BootstrapError;
use storm_node::stormd::Opts;
use storm_node::{stormd, Config, LaunchError};

fn main() -> Result<(), BootstrapError<LaunchError>> {
    println!("stored: storage microservice");

    let mut opts = Opts::parse();
    trace!("Command-line arguments: {:?}", opts);
    opts.process();
    trace!("Processed arguments: {:?}", opts);

    let config: Config<stormd::Config> = opts.clone().into();
    trace!("Daemon configuration: {:?}", config);
    debug!("MSG socket {}", config.msg_endpoint);
    debug!("CTL socket {}", config.ctl_endpoint);
    debug!("RPC socket {}", config.rpc_endpoint);
    debug!("STORM socket {}", config.ext_endpoint);
    debug!("STORE socket {}", config.store_endpoint);
    debug!("CHAT socket {}", config.chat_endpoint);

    /*
    use self::internal::ResultExt;
    let (config_from_file, _) =
        internal::Config::custom_args_and_optional_files(std::iter::empty::<
            &str,
        >())
        .unwrap_or_exit();
     */

    debug!("Starting runtime ...");
    stormd::run(config).expect("running stromd runtime");

    unreachable!()
}
