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

use internet2::addr::{NodeId, PartialSocketAddr, ServiceAddr};
use lnp_rpc::LNP_NODE_RPC_ENDPOINT;
use stens::AsciiString;
use store_rpc::STORED_RPC_ENDPOINT;
use storm::ContainerId;
use storm_rpc::{CHATD_RPC_ENDPOINT, STORM_NODE_RPC_ENDPOINT};

/// Command-line tool for working with store daemon
#[derive(Parser, Clone, PartialEq, Eq, Debug)]
#[clap(name = "storm-cli", bin_name = "storm-cli", author, version)]
pub struct Opts {
    /// ZMQ socket for connecting Storm node RPC interface.
    ///
    /// Socket can be either TCP address in form of `<ipv4 | ipv6>:<port>` – or a path
    /// to an IPC file.
    #[clap(
        short = 'S',
        long = "storm",
        global = true,
        default_value = STORM_NODE_RPC_ENDPOINT,
        env = "STORM_NODE_RPC_ENDPOINT"
    )]
    pub storm_endpoint: ServiceAddr,

    /// ZMQ socket for connecting storage daemon.
    ///
    /// Socket can be either TCP address in form of `<ipv4 | ipv6>:<port>` – or a path
    /// to an IPC file.
    #[clap(
        long = "store",
        global = true,
        env = "STORED_RPC_ENDPOINT",
        default_value = STORED_RPC_ENDPOINT,
    )]
    pub store_endpoint: ServiceAddr,

    /// ZMQ socket for chat daemon PUB/SUB API.
    ///
    /// Socket can be either TCP address in form of `<ipv4 | ipv6>:<port>` – or a path
    /// to an IPC file.
    #[clap(
        short = 'C',
        long = "chat",
        global = true,
        env = "CHATD_RPC_ENDPOINT",
        default_value = CHATD_RPC_ENDPOINT,
    )]
    pub radio_endpoint: ServiceAddr,

    /// ZMQ socket for connecting LNP node RPC interface.
    ///
    /// Socket can be either TCP address in form of `<ipv4 | ipv6>:<port>` – or a path
    /// to an IPC file.
    #[clap(
        short = 'L',
        long = "lnp",
        global = true,
        default_value = LNP_NODE_RPC_ENDPOINT,
        env = "LNP_NODE_RPC_ENDPOINT"
    )]
    pub lnp_endpoint: ServiceAddr,

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
    /// Listen for the incoming chat messages from a remote peer.
    #[display("chat-listen")]
    ChatListen {
        /// Remote node address to force connection (re)establishment
        #[clap(long)]
        connect: Option<PartialSocketAddr>,

        /// Remote node id (public key).
        peer: NodeId,
    },

    /// Send typed-in messages to another peer.
    #[display("chat-send")]
    ChatSend {
        /// Remote node address to force connection (re)establishment
        #[clap(long)]
        connect: Option<PartialSocketAddr>,

        /// Remote node id (public key).
        peer: NodeId,
    },

    /// Convert on-disk file into a container in the Store database.
    FileContainerize {
        /// MIME file type
        #[clap(short, long, default_value = "application/octet-stream")]
        mime: AsciiString,

        /// Local file for containerization.
        path: PathBuf,

        /// Information about the container
        #[clap()]
        info: Option<String>,
    },

    /// Assemble a file from a Store database-present container and save as a file.
    FileAssemble {
        /// ID of the container to assemble into a file.
        container_id: ContainerId,

        /// Path and filename to save the file.
        path: PathBuf,
    },
}
