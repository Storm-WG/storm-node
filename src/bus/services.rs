// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::str::FromStr;

use internet2::addr::NodeAddr;
use internet2::TypedEnum;
use lnp2p::bifrost;
use lnp2p::bifrost::BifrostApp;
use lnp_rpc::{ClientId, ServiceName};
use microservices::esb;
use storm::{p2p, StormApp};
use storm_ext::ExtMsg;
use storm_rpc::RpcMsg;
use strict_encoding::{strict_deserialize, strict_serialize};

use crate::bus::{BusMsg, CtlMsg};

pub(crate) type Endpoints = esb::EndpointList<ServiceBus>;

pub type DaemonId = u64;

/// Identifiers of daemons participating in LNP Node
#[derive(Clone, PartialEq, Eq, Hash, Debug, Display, From, StrictEncode, StrictDecode)]
pub enum ServiceId {
    #[display("stormd")]
    #[strict_encoding(value = 0x24)] // This mimics Bifrost LNP storm service id
    BifrostApp(BifrostApp),

    #[display("app<{0}>")]
    #[strict_encoding(value = 0x41)]
    StormApp(StormApp),

    #[display("client<{0}>")]
    #[strict_encoding(value = 2)]
    Client(ClientId),

    #[display("lnpd")]
    #[strict_encoding(value = 0x20)]
    Lnp,

    #[display("peerd<{0}>")]
    #[from]
    #[strict_encoding(value = 0x21)]
    Peer(NodeAddr),

    #[display("tansferd<{0}>")]
    #[strict_encoding(value = 0x42)]
    Transfer(DaemonId),

    #[display("other<{0}>")]
    #[strict_encoding(value = 0xFF)]
    Other(ServiceName),
}

impl ServiceId {
    pub fn storm_broker() -> ServiceId { ServiceId::BifrostApp(BifrostApp::Storm) }
}

impl esb::ServiceAddress for ServiceId {}

impl From<ServiceId> for Vec<u8> {
    fn from(daemon_id: ServiceId) -> Self {
        strict_serialize(&daemon_id).expect("Memory-based encoding does not fail")
    }
}

impl From<Vec<u8>> for ServiceId {
    fn from(vec: Vec<u8>) -> Self {
        strict_deserialize(&vec).unwrap_or_else(|_| {
            ServiceId::Other(
                ServiceName::from_str(&String::from_utf8_lossy(&vec))
                    .expect("ClientName conversion never fails"),
            )
        })
    }
}

/// Service buses used for inter-daemon communication
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Display)]
pub(crate) enum ServiceBus {
    /// Storm application messaging
    #[display("EXT")]
    Ext,

    /// RPC interface, from client to node
    #[display("RPC")]
    Rpc,

    /// CTL interface, from daemon to daemon control messages
    #[display("CTL")]
    Ctl,

    /// LN P2P (Bifrost) message bus
    #[display("MSG")]
    Msg,
}

impl esb::BusId for ServiceBus {
    type Address = ServiceId;
}

pub(crate) trait Responder
where
    Self: esb::Handler<ServiceBus>,
    esb::Error<ServiceId>: From<Self::Error>,
{
    #[inline]
    fn send_p2p(
        &self,
        endpoints: &mut Endpoints,
        remote_peer: NodeAddr,
        message: impl Into<p2p::Messages>,
    ) -> Result<(), esb::Error<ServiceId>> {
        let message = message.into().serialize();
        let message = BusMsg::Bifrost(bifrost::Messages::Message(bifrost::Msg {
            app: BifrostApp::Storm,
            payload: Box::from(message),
        }));
        endpoints.send_to(ServiceBus::Msg, self.identity(), ServiceId::Peer(remote_peer), message)
    }

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
    fn send_ctl(
        &self,
        endpoints: &mut Endpoints,
        service_id: ServiceId,
        message: impl Into<CtlMsg>,
    ) -> Result<(), esb::Error<ServiceId>> {
        endpoints.send_to(ServiceBus::Ctl, self.identity(), service_id, BusMsg::Ctl(message.into()))
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
            ServiceId::StormApp(app_id.into()),
            BusMsg::Ext(message.into()),
        )
    }
}
