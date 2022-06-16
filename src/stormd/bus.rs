// Storm node providing distributed storage & messaging for lightning network.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use lnp_rpc::ServiceId;
use microservices::{esb, rpc};
use storm_rpc::RpcMsg;

/// Service buses used for inter-daemon communication
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Display)]
pub enum ServiceBus {
    /// RPC interface, from client to node
    #[display("RPC")]
    Rpc,

    /// LN P2P (Bifrost) message bus
    #[display("MSG")]
    Msg,
}

impl esb::BusId for ServiceBus {
    type Address = ServiceId;
}

/// Service controller messages
#[derive(Clone, Debug, Display, From, Api)]
#[api(encoding = "strict")]
#[display(inner)]
pub enum BusMsg {
    /// Bifrost P2P messages
    #[api(type = 3)]
    #[from]
    Birfost(lnp2p::bifrost::Messages),

    /// RPC requests
    #[api(type = 4)]
    #[display(inner)]
    #[from]
    Rpc(RpcMsg),
}

impl rpc::Request for BusMsg {}
