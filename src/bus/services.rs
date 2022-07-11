// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use internet2::addr::NodeId;
use internet2::TypedEnum;
use lnp2p::bifrost;
use lnp2p::bifrost::BifrostApp;
use microservices::esb::ClientId;
use microservices::util::OptionDetails;
use microservices::{esb, rpc};
use storm::{p2p, StormApp};
use storm_ext::ExtMsg;
use storm_rpc::{RadioMsg, RpcMsg, ServiceId};

use crate::bus::{BusMsg, CtlMsg};

pub(crate) type Endpoints = esb::EndpointList<ServiceBus>;

pub type DaemonId = u64;

/// Service buses used for inter-daemon communication
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Display)]
pub(crate) enum ServiceBus {
    /// Storm application messaging
    #[display("EXT")]
    Storm,

    /// RPC interface, from client to node
    #[display("RPC")]
    Rpc,

    /// CTL interface, from daemon to daemon control messages
    #[display("CTL")]
    Ctl,

    /// LN P2P (Bifrost) message bus
    #[display("MSG")]
    Msg,

    /// Pub/sub bus used for chat daemon
    #[display("CHAT")]
    Chat,
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
    fn send_p2p_reporting_client(
        &self,
        endpoints: &mut Endpoints,
        client_id: ClientId,
        client_message: impl Into<OptionDetails>,
        remote_id: NodeId,
        message: impl Into<p2p::Messages>,
    ) {
        let client_message = client_message.into();
        // We have nobody to report the failure to
        let _ = match self.send_p2p(endpoints, remote_id, message) {
            Ok(_) => {
                if let Some(client_message) = client_message.0 {
                    self.send_rpc(endpoints, client_id, RpcMsg::Progress(client_message))
                } else {
                    Ok(())
                }
            }
            Err(err) => {
                let failure = rpc::Failure {
                    code: rpc::FailureCode::Transport,
                    info: format!("{}", err),
                };
                self.send_rpc(endpoints, client_id, failure)
            }
        }
        .map_err(|_| warn!("client {} is disconnected", client_id));
    }

    #[inline]
    fn send_p2p(
        &self,
        endpoints: &mut Endpoints,
        remote_id: NodeId,
        message: impl Into<p2p::Messages>,
    ) -> Result<(), esb::Error<ServiceId>> {
        let payload = message.into().serialize();
        let message = BusMsg::Bifrost(bifrost::Messages::Message(bifrost::Msg {
            app: BifrostApp::Storm,
            payload: Box::from(payload),
        }));
        endpoints.send_to(ServiceBus::Msg, self.identity(), ServiceId::Peer(remote_id), message)
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
        app_id: Option<StormApp>,
        message: impl Into<ExtMsg>,
    ) -> Result<(), esb::Error<ServiceId>> {
        endpoints.send_to(
            ServiceBus::Storm,
            self.identity(),
            app_id.map(ServiceId::StormApp).unwrap_or(ServiceId::stormd()),
            BusMsg::Storm(message.into()),
        )
    }

    #[inline]
    fn send_radio(
        &self,
        endpoints: &mut Endpoints,
        message: impl Into<RadioMsg>,
    ) -> Result<(), esb::Error<ServiceId>> {
        endpoints.send_to(
            ServiceBus::Chat,
            self.identity(),
            // Not sure this will work though
            ServiceId::stormd(),
            BusMsg::Chat(message.into()),
        )
    }
}
