// Storm node providing distributed storage & messaging for lightning network.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use lnp_rpc::{ClientId, ServiceId};
use microservices::{esb, rpc};
use storm::StormApp;
use storm_ext::ExtMsg;
use storm_rpc::RpcMsg;

pub(crate) type Endpoints = esb::EndpointList<ServiceBus>;

/// Service buses used for inter-daemon communication
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Display)]
pub(crate) enum ServiceBus {
    /// Storm application messaging
    #[display("EXT")]
    Ext,

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
pub(crate) enum BusMsg {
    /// Bifrost P2P messages
    #[api(type = 3)]
    #[from]
    Birfost(lnp2p::bifrost::Messages),

    /// RPC requests
    #[api(type = 4)]
    #[display(inner)]
    #[from]
    Rpc(RpcMsg),

    /// Storm node <-> application extensions messaging
    #[api(type = 5)]
    #[display(inner)]
    #[from]
    Ext(ExtMsg),
}

impl rpc::Request for BusMsg {}

pub(crate) trait Responder
where
    Self: esb::Handler<ServiceBus>,
    esb::Error<ServiceId>: From<Self::Error>,
{
    #[inline]
    fn send_rpc(
        &self,
        endpoints: &mut Endpoints,
        client_id: ClientId,
        message: impl Into<RpcMsg>,
    ) -> Result<(), esb::Error<ServiceId>> {
        endpoints.send_to(
            ServiceBus::Rpc,
            self.identity(),
            ServiceId::Client(client_id),
            BusMsg::Rpc(message.into()),
        )
    }

    #[inline]
    fn send_ext(
        &self,
        endpoints: &mut Endpoints,
        app_id: StormApp,
        message: impl Into<ExtMsg>,
    ) -> Result<(), esb::Error<ServiceId>> {
        endpoints.send_to(
            ServiceBus::Rpc,
            self.identity(),
            ServiceId::Layer3App(app_id.into()),
            BusMsg::Ext(message.into()),
        )
    }
}