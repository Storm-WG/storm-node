// Storm node providing distributed storage & messaging for lightning network.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::path::PathBuf;

use clap::{Parser, ValueHint};
use internet2::addr::ServiceAddr;
use microservices::shell::shell_setup;
use storm_ext::STORM_NODE_EXT_ENDPOINT;
use storm_rpc::STORM_NODE_RPC_ENDPOINT;

#[cfg(any(target_os = "linux"))]
pub const STORM_NODE_DATA_DIR: &str = "~/.storm";
#[cfg(any(target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
pub const STORM_NODE_DATA_DIR: &str = "~/.storm";
#[cfg(target_os = "macos")]
pub const STORM_NODE_DATA_DIR: &str = "~/Library/Application Support/Storm Node";
#[cfg(target_os = "windows")]
pub const STORM_NODE_DATA_DIR: &str = "~\\AppData\\Local\\Storm Node";
#[cfg(target_os = "ios")]
pub const STORM_NODE_DATA_DIR: &str = "~/Documents";
#[cfg(target_os = "android")]
pub const STORM_NODE_DATA_DIR: &str = ".";

pub const STORM_NODE_CTL_ENDPOINT: &str = "{data_dir}/ctl";

pub const STORM_NODE_CONFIG: &str = "{data_dir}/stormd.toml";

/// Command-line arguments
#[derive(Parser)]
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[clap(author, version, name = "stormd", about = "storm node managing service")]
pub struct Opts {
    /// Set verbosity level.
    ///
    /// Can be used multiple times to increase verbosity
    #[clap(short, long, global = true, parse(from_occurrences))]
    pub verbose: u8,

    /// Data directory path.
    ///
    /// Path to the directory that contains stored data, and where ZMQ RPC
    /// socket files are located
    #[clap(
        short,
        long,
        global = true,
        default_value = STORM_NODE_DATA_DIR,
        env = "STORM_NODE_DATA_DIR",
        value_hint = ValueHint::DirPath
    )]
    pub data_dir: PathBuf,

    /// Path for the configuration file.
    ///
    /// NB: Command-line options override configuration file values.
    #[clap(
        short,
        long,
        global = true,
        env = "STORM_NODE_CONFIG",
        value_hint = ValueHint::FilePath
    )]
    pub config: Option<PathBuf>,

    /// ZMQ socket for peer message bus used to communicate with LNP node peerd
    /// service.
    ///
    /// A user needs to specify this socket usually if it likes to distribute daemons
    /// over different server instances. In this case all daemons within the same node
    /// must use the same socket address.
    ///
    /// Socket can be either TCP address in form of `<ipv4 | ipv6>:<port>` – or a path
    /// to an IPC file.
    ///
    /// Defaults to `msg` file inside `--data-dir` directory.
    #[clap(
        short = 'M',
        long = "msg",
        env = "LNP_NODE_MSG_ENDPOINT",
        value_hint = ValueHint::FilePath
    )]
    pub msg_endpoint: ServiceAddr,

    /// ZMQ socket for internal service control bus.
    ///
    /// A user needs to specify this socket usually if it likes to distribute daemons
    /// over different server instances. In this case all daemons within the same node
    /// must use the same socket address.
    ///
    /// Socket can be either TCP address in form of `<ipv4 | ipv6>:<port>` – or a path
    /// to an IPC file.
    ///
    /// Defaults to `ctl` file inside `--data-dir` directory, unless `--threaded-daemons`
    /// is specified; in that cases parameter in-memory communication protocol is used
    /// by default (see ZMQ inproc socket specification).
    #[clap(
        short = 'X',
        long = "ctl",
        global = true,
        env = "STORM_NODE_CTL_ENDPOINT",
        default_value = STORM_NODE_CTL_ENDPOINT,
        value_hint = ValueHint::FilePath
    )]
    pub ctl_endpoint: ServiceAddr,

    /// ZMQ socket name/address for Storm Node client-server RPC API.
    ///
    /// Socket can be either TCP address in form of `<ipv4 | ipv6>:<port>` – or a path
    /// to an IPC file.
    #[clap(
        short = 'R',
        long,
        env = "STORM_NODE_RPC_ENDPOINT",
        value_hint = ValueHint::FilePath,
        default_value = STORM_NODE_RPC_ENDPOINT
    )]
    pub rpc_endpoint: ServiceAddr,

    /// ZMQ socket name/address for Storm extensions interface, used to handle application-specific
    /// messages to and from extension daemons, connected to this bus.
    #[clap(
        short = 'E',
        long,
        env = "STORM_NODE_EXT_ENDPOINT",
        value_hint = ValueHint::FilePath,
        default_value = STORM_NODE_EXT_ENDPOINT
    )]
    pub ext_endpoint: ServiceAddr,

    /// Spawn daemons as threads and not processes
    #[clap(short = 'T', long = "threaded")]
    pub threaded_daemons: bool,
}

impl Opts {
    pub fn process(&mut self) {
        shell_setup(
            self.verbose,
            [
                &mut self.msg_endpoint,
                &mut self.ctl_endpoint,
                &mut self.rpc_endpoint,
                &mut self.ext_endpoint,
            ],
            &mut self.data_dir,
            &[],
        );
    }
}
