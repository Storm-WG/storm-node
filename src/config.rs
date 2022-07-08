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

use internet2::addr::ServiceAddr;

/// Final configuration resulting from data contained in config file environment
/// variables and command-line options. For security reasons node key is kept
/// separately.
#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[display(Debug)]
pub struct Config {
    /// Data location
    pub data_dir: PathBuf,

    /// ZMQ socket for lightning peer network message bus
    pub msg_endpoint: ServiceAddr,

    /// ZMQ socket for internal service control bus
    pub ctl_endpoint: ServiceAddr,

    /// ZMQ socket for client-service RCP API.
    pub rpc_endpoint: ServiceAddr,

    /// ZMQ socket for inter-storm app messaging.
    pub ext_endpoint: ServiceAddr,

    /// ZMQ socket for Store service RPC.
    pub store_endpoint: ServiceAddr,

    /// Indicates whether deamons should be spawned as threads (true) or as child processes (false)
    pub threaded: bool,
}
