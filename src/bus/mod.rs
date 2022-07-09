// Storm node providing distributed storage & messaging for lightning network.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

mod services;
mod ctl;

use lnp2p::bifrost;
use microservices::rpc;
use storm_ext::ExtMsg;
use storm_rpc::RpcMsg;

pub use self::ctl::CtlMsg;
pub(crate) use self::services::{DaemonId, Endpoints, Responder, ServiceBus};

/// Service controller messages
#[derive(Clone, Debug, Display, From, Api)]
#[api(encoding = "strict")]
#[display(inner)]
pub(crate) enum BusMsg {
    /// Bifrost P2P messages
    #[api(type = 3)]
    #[from]
    Bifrost(bifrost::Messages),

    /// RPC requests
    #[api(type = 4)]
    #[display(inner)]
    #[from]
    Rpc(RpcMsg),

    /// Storm node <-> application extensions messaging
    #[api(type = 5)]
    #[display(inner)]
    #[from]
    Storm(ExtMsg),

    /// CTL requests
    #[api(type = 6)]
    #[display(inner)]
    #[from]
    Ctl(CtlMsg),
}

impl rpc::Request for BusMsg {}
