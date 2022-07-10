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

//! Command-line interface to Storm Node

#[macro_use]
extern crate amplify;
#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;

mod command;
mod opts;

use clap::Parser;
use internet2::addr::ServiceAddr;
use microservices::cli::LogStyle;
use microservices::shell::LogLevel;

pub use crate::opts::{Command, Opts};

fn main() {
    println!("storm-cli: command-line tool for working with Storm node");

    let mut opts = Opts::parse();
    LogLevel::from_verbosity_flag_count(opts.verbose).apply();
    trace!("Command-line arguments: {:#?}", &opts);

    let storm_endpoint = &mut opts.storm_endpoint;
    let radio_endpoint = &mut opts.radio_endpoint;
    if let ServiceAddr::Ipc(ref mut path) = storm_endpoint {
        *path = shellexpand::tilde(path).to_string();
    }
    if let ServiceAddr::Ipc(ref mut path) = radio_endpoint {
        *path = shellexpand::tilde(path).to_string();
    }
    debug!("STORM RPC socket {}", storm_endpoint);
    let mut storm_client =
        storm_rpc::Client::with(storm_endpoint.clone(), radio_endpoint.clone(), s!("storm-cli"))
            .expect("Error initializing Storm client");

    let lnp_endpoint = &mut opts.lnp_endpoint;
    if let ServiceAddr::Ipc(ref mut path) = lnp_endpoint {
        *path = shellexpand::tilde(path).to_string();
    }
    debug!("LNP RPC socket {}", lnp_endpoint);
    let mut lnp_client =
        lnp_rpc::Client::with(lnp_endpoint.clone()).expect("Error initializing LNP client");

    trace!("Executing command: {}", opts.command);
    opts.exec(&mut storm_client, &mut lnp_client)
        .unwrap_or_else(|err| eprintln!("{} {}\n", "Error:".err(), err.err_details()));
}
