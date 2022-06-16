// Storm node providing distributed storage & messaging for lightning network.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

// TODO: Remove this file once rust-microservices will support different message
//       types per bus in ESB controller

use lnp_rpc::ServiceId;
use microservices::{esb, rpc};

/// Service buses used for inter-daemon communication
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Display)]
pub enum ServiceBus {
    /// LN P2P message bus
    #[display("MSG")]
    Msg,
}

impl esb::BusId for ServiceBus {
    type Address = ServiceId;
}

/// Service bus messages wrapping all other message types
#[derive(Clone, Debug, Display, From, Api)]
#[api(encoding = "strict")]
#[display(inner)]
pub enum BusMsg {
    /// Wrapper for Bifrost P2P messages to be transmitted over control bus
    #[api(type = 3)]
    #[from]
    Birfost(lnp2p::bifrost::Messages),
}

impl rpc::Request for BusMsg {}
