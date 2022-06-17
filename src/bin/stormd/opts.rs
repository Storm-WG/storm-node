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
use storm_app::STORM_NODE_APP_ENDPOINT;
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

    /// ZMQ socket for connecting LNP node message bus.
    ///
    /// A user needs to specify this socket usually if it likes to distribute daemons
    /// over different server instances. In this case all daemons within the same node
    /// must use the same socket address.
    ///
    /// Socket can be either TCP address in form of `<ipv4 | ipv6>:<port>` – or a path
    /// to an IPC file.
    #[clap(short, long, env = "LNP_NODE_MSG_SOCKET", value_hint = ValueHint::FilePath)]
    pub msg_endpoint: ServiceAddr,

    /// ZMQ socket name/address for storm node RPC interface.
    ///
    /// Internal interface for control PRC protocol communications.
    #[clap(
        short = 'x',
        long,
        env = "STORM_NODE_RPC_ENDPOINT",
        value_hint = ValueHint::FilePath,
        default_value = STORM_NODE_RPC_ENDPOINT
    )]
    pub rpc_endpoint: ServiceAddr,

    /// ZMQ socket name/address for storm node RPC interface.
    ///
    /// Internal interface for control PRC protocol communications.
    #[clap(
        short,
        long,
        env = "STORM_NODE_APP_ENDPOINT",
        value_hint = ValueHint::FilePath,
        default_value = STORM_NODE_APP_ENDPOINT
    )]
    pub app_endpoint: ServiceAddr,
}
