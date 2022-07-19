// Storm node providing distributed storage & messaging for lightning network.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::fmt::Debug;
use std::path::PathBuf;

use internet2::addr::ServiceAddr;

use crate::opts::Options;

/// Final configuration resulting from data contained in config file environment
/// variables and command-line options. For security reasons node key is kept
/// separately.
#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[display(Debug)]
pub struct Config<Ext = ()>
where Ext: Clone + Eq + Debug
{
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

    /// ZMQ socket for chat daemon PUB/SUB API.
    pub chat_endpoint: ServiceAddr,

    /// Daemon-specific config extensions
    pub ext: Ext,
}

impl<Ext> Config<Ext>
where Ext: Clone + Eq + Debug
{
    pub fn with<Orig>(orig: Config<Orig>, ext: Ext) -> Self
    where Orig: Clone + Eq + Debug {
        Config::<Ext> {
            data_dir: orig.data_dir,
            rpc_endpoint: orig.rpc_endpoint,
            msg_endpoint: orig.msg_endpoint,
            ext_endpoint: orig.ext_endpoint,
            ctl_endpoint: orig.ctl_endpoint,
            store_endpoint: orig.store_endpoint,
            chat_endpoint: orig.chat_endpoint,
            ext,
        }
    }
}

#[cfg(feature = "server")]
impl<Opt> From<Opt> for Config<Opt::Conf>
where
    Opt: Options,
    Opt::Conf: Clone + Eq + Debug,
{
    fn from(opt: Opt) -> Self {
        let opts = opt.shared();

        Config {
            data_dir: opts.data_dir.clone(),
            rpc_endpoint: opts.rpc_endpoint.clone(),
            msg_endpoint: opts.msg_endpoint.clone(),
            ext_endpoint: opts.ext_endpoint.clone(),
            store_endpoint: opts.store_endpoint.clone(),
            chat_endpoint: opts.chat_endpoint.clone(),
            ctl_endpoint: opts.ctl_endpoint.clone(),
            ext: opt.config(),
        }
    }
}
