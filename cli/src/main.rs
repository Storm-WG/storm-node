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
use colored::Colorize;
use internet2::addr::ServiceAddr;
use microservices::shell::{Exec, LogLevel};
use storm_rpc::client::Client;

pub use crate::opts::{Command, Opts};

fn main() {
    println!("storm-cli: command-line tool for working with Storm node");

    let opts = Opts::parse();
    LogLevel::from_verbosity_flag_count(opts.verbose).apply();
    trace!("Command-line arguments: {:#?}", &opts);

    let connect = &mut opts.connect;
    if let ServiceAddr::Ipc(ref mut path) = connect {
        *path = shellexpand::tilde(path).to_string();
    }
    debug!("RPC socket {}", connect);

    let mut client = Client::with(connect).expect("Error initializing client");

    trace!("Executing command: {}", opts.command);
    opts.command.exec(&mut client).unwrap_or_else(|err| eprintln!("{}", err));
}
