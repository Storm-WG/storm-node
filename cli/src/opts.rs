// Storm node providing distributed storage & messaging for lightning network.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use internet2::addr::{NodeAddr, ServiceAddr};
use lnp_rpc::LNP_NODE_RPC_ENDPOINT;
use storm_rpc::STORM_NODE_RPC_ENDPOINT;

/// Command-line tool for working with store daemon
#[derive(Parser, Clone, PartialEq, Eq, Debug)]
#[clap(name = "storm-cli", bin_name = "storm-cli", author, version)]
pub struct Opts {
    /// ZMQ socket for connecting Storm node RPC interface.
    ///
    /// Socket can be either TCP address in form of `<ipv4 | ipv6>:<port>` – or a path
    /// to an IPC file.
    ///
    /// Defaults to `127.0.0.1:64964`.
    #[clap(
        short = 'R',
        long = "rpc",
        global = true,
        default_value = STORM_NODE_RPC_ENDPOINT,
        env = "STORM_NODE_RPC_ENDPOINT"
    )]
    pub connect: ServiceAddr,

    /// ZMQ socket for connecting LNP node RPC interface.
    ///
    /// Socket can be either TCP address in form of `<ipv4 | ipv6>:<port>` – or a path
    /// to an IPC file.
    ///
    /// Defaults to `127.0.0.1:62962`.
    #[clap(
        short = 'L',
        long = "lnp",
        global = true,
        default_value = LNP_NODE_RPC_ENDPOINT,
        env = "LNP_NODE_RPC_ENDPOINT"
    )]
    pub lnp_rpc: ServiceAddr,

    /// Set verbosity level.
    ///
    /// Can be used multiple times to increase verbosity.
    #[clap(short, long, global = true, parse(from_occurrences))]
    pub verbose: u8,

    /// Command to execute
    #[clap(subcommand)]
    pub command: Command,
}

/// Command-line commands:
#[derive(Subcommand, Clone, PartialEq, Eq, Debug, Display)]
pub enum Command {
    #[display("chat-listen")]
    ChatListen { peer: NodeAddr },

    #[display("chat-send")]
    ChatSend { peer: NodeAddr },
}
