// Storm node providing distributed storage & messaging for lightning network.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::collections::BTreeSet;
use std::ops::Deref;

use internet2::addr::NodeAddr;
use internet2::{Unmarshall, ZmqSocketType};
use lnp2p::bifrost;
use lnp2p::bifrost::{BifrostApp, Messages as LnMsg};
use lnp_rpc::ClientId;
use microservices::error::BootstrapError;
use microservices::esb;
use microservices::esb::{EndpointList, Error};
use microservices::node::TryService;
use storm::p2p::{Messages, STORM_P2P_UNMARSHALLER};
use storm::StormApp;
use storm_ext::ExtMsg;
use storm_rpc::RpcMsg;

use crate::bus::{BusMsg, Endpoints, Responder, ServiceBus, ServiceId};
use crate::{Config, DaemonError, LaunchError};

pub fn run(config: Config) -> Result<(), BootstrapError<LaunchError>> {
    let msg_endpoint = config.msg_endpoint.clone();
    let rpc_endpoint = config.rpc_endpoint.clone();
    let ctl_endpoint = config.ctl_endpoint.clone();
    let ext_endpoint = config.ext_endpoint.clone();
    let runtime = Runtime::init(config)?;

    debug!("Connecting to service bus {}", msg_endpoint);
    let controller = esb::Controller::with(
        map! {
            ServiceBus::Ext => esb::BusConfig::with_addr(
                ext_endpoint,
                ZmqSocketType::RouterBind,
                Some(ServiceId::storm_broker())
            ),
            ServiceBus::Ctl => esb::BusConfig::with_addr(
                ctl_endpoint,
                ZmqSocketType::RouterBind,
                Some(ServiceId::storm_broker())
            ),
            ServiceBus::Msg => esb::BusConfig::with_addr(
                msg_endpoint,
                ZmqSocketType::RouterConnect,
                Some(ServiceId::storm_broker())
            ),
            ServiceBus::Rpc => esb::BusConfig::with_addr(
                rpc_endpoint,
                ZmqSocketType::Rep,
                None
            )
        },
        runtime,
    )
    .map_err(|_| LaunchError::NoLnpdConnection)?;

    controller.run_or_panic("stormd");

    unreachable!()
}

pub struct Runtime {
    registered_apps: BTreeSet<StormApp>,
}

impl Runtime {
    pub fn init(config: Config) -> Result<Self, BootstrapError<LaunchError>> {
        // debug!("Initializing storage provider {:?}", config.storage_conf());
        // let storage = storage::FileDriver::with(config.storage_conf())?;

        info!("Stormd runtime started successfully");

        Ok(Self {
            registered_apps: empty!(),
        })
    }
}

impl Responder for Runtime {}

impl esb::Handler<ServiceBus> for Runtime {
    type Request = BusMsg;
    type Error = DaemonError;

    fn identity(&self) -> ServiceId { ServiceId::storm_broker() }

    fn handle(
        &mut self,
        endpoints: &mut EndpointList<ServiceBus>,
        bus_id: ServiceBus,
        source: ServiceId,
        request: Self::Request,
    ) -> Result<(), Self::Error> {
        match (bus_id, request, source) {
            (ServiceBus::Msg, BusMsg::Bifrost(msg), ServiceId::Peer(remote_peer)) => {
                self.handle_p2p(endpoints, remote_peer, msg)
            }
            (ServiceBus::Ext, BusMsg::Ext(msg), source) => self.handle_app(endpoints, source, msg),
            (ServiceBus::Rpc, BusMsg::Rpc(msg), ServiceId::Client(client_id)) => {
                self.handle_rpc(endpoints, client_id, msg)
            }
            (bus, msg, _) => Err(DaemonError::wrong_esb_msg(bus, &msg)),
        }
    }

    fn handle_err(
        &mut self,
        _endpoints: &mut EndpointList<ServiceBus>,
        _error: Error<ServiceId>,
    ) -> Result<(), Self::Error> {
        // We do nothing and do not propagate error; it's already being reported
        // with `error!` macro by the controller. If we propagate error here
        // this will make whole daemon panic
        Ok(())
    }
}

impl Runtime {
    fn handle_p2p(
        &mut self,
        endpoints: &mut Endpoints,
        remote_peer: NodeAddr,
        message: LnMsg,
    ) -> Result<(), DaemonError> {
        if let LnMsg::Message(bifrost::Msg {
            app: BifrostApp::Storm,
            payload,
        }) = &message
        {
            let mesg = STORM_P2P_UNMARSHALLER.unmarshall(&**payload)?.deref().clone();
            match &mesg {
                // Messages we process ourselves
                Messages::ListApps => {
                    self.send_p2p(
                        endpoints,
                        remote_peer,
                        Messages::ActiveApps(self.registered_apps.clone()),
                    )?;
                }

                // Messages we forward to the remote peer
                Messages::ActiveApps(_) => {}
                Messages::ListTopics(_) => {}
                Messages::AppTopics(_) => {}
                Messages::ProposeTopic(_) => {}
                Messages::Post(_) => {}
                Messages::Read(_) => {}
                Messages::Decline(_) => {}
                Messages::Accept(_) => {}
                Messages::PullContainer(_) => {}
                Messages::PushContainer(_) => {}
                Messages::Reject(_) => {}
                Messages::PullChunk(_) => {}
                Messages::PushChunk(_) => {}
                _ => {}
            }
        } else {
            error!("Request is not supported by the RPC interface");
            return Err(DaemonError::wrong_esb_msg(ServiceBus::Rpc, &message));
        }
        Ok(())
    }

    fn handle_rpc(
        &mut self,
        endpoints: &mut Endpoints,
        client_id: ClientId,
        message: RpcMsg,
    ) -> Result<(), DaemonError> {
        match message {
            wrong_msg => {
                error!("Request is not supported by the RPC interface");
                return Err(DaemonError::wrong_esb_msg(ServiceBus::Rpc, &wrong_msg));
            }
        }

        Ok(())
    }

    fn handle_app(
        &mut self,
        endpoints: &mut Endpoints,
        source: ServiceId,
        message: ExtMsg,
    ) -> Result<(), DaemonError> {
        match &message {
            wrong_msg => {
                error!("Request is not supported by the APP interface");
                return Err(DaemonError::wrong_esb_msg(ServiceBus::Ext, wrong_msg));
            }
        }

        Ok(())
    }
}
