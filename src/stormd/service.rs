// Storm node providing distributed storage & messaging for lightning network.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::ops::Deref;

use internet2::addr::NodeAddr;
use internet2::{CreateUnmarshaller, Unmarshall, Unmarshaller, ZmqSocketType};
use lnp2p::bifrost;
use lnp2p::bifrost::{BifrostApp, Messages as LnMsg};
use lnp_rpc::{ClientId, ServiceId};
use microservices::error::BootstrapError;
use microservices::esb;
use microservices::esb::{EndpointList, Error};
use microservices::node::TryService;
use storm::p2p::{Messages as AppMsg, StormMesg, STORM_P2P_UNMARSHALLER};
use storm_rpc::RpcMsg;

use crate::bus::{BusMsg, Endpoints, Responder, ServiceBus};
use crate::{Config, DaemonError, LaunchError};

pub fn run(config: Config) -> Result<(), BootstrapError<LaunchError>> {
    let msg_endpoint = config.msg_endpoint.clone();
    let rpc_endpoint = config.rpc_endpoint.clone();
    let app_endpoint = config.app_endpoint.clone();
    let runtime = Runtime::init(config)?;

    debug!("Connecting to service bus {}", msg_endpoint);
    let controller = esb::Controller::with(
        map! {
            ServiceBus::App => esb::BusConfig::with_addr(
                app_endpoint,
                ZmqSocketType::RouterBind,
                Some(ServiceId::MsgApp(BifrostApp::Storm))
            ),
            ServiceBus::Msg => esb::BusConfig::with_addr(
                msg_endpoint,
                ZmqSocketType::RouterConnect,
                None
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
    /// Original configuration object
    pub(crate) config: Config,

    /// Unmarshaller instance used for parsing RPC request
    pub(crate) unmarshaller: Unmarshaller<BusMsg>,
}

impl Runtime {
    pub fn init(config: Config) -> Result<Self, BootstrapError<LaunchError>> {
        // debug!("Initializing storage provider {:?}", config.storage_conf());
        // let storage = storage::FileDriver::with(config.storage_conf())?;

        info!("Stormd runtime started successfully");

        Ok(Self {
            config,
            unmarshaller: BusMsg::create_unmarshaller(),
        })
    }
}

impl Responder for Runtime {}

impl esb::Handler<ServiceBus> for Runtime {
    type Request = BusMsg;
    type Error = DaemonError;

    fn identity(&self) -> ServiceId { ServiceId::MsgApp(BifrostApp::Storm) }

    fn handle(
        &mut self,
        endpoints: &mut EndpointList<ServiceBus>,
        bus_id: ServiceBus,
        source: ServiceId,
        request: Self::Request,
    ) -> Result<(), Self::Error> {
        match (bus_id, request, source) {
            (ServiceBus::Msg, BusMsg::Birfost(msg), ServiceId::Peer(remote_peer)) => {
                self.handle_p2p(endpoints, remote_peer, msg)
            }
            (ServiceBus::App, BusMsg::App(msg), source) => self.handle_app(endpoints, source, msg),
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
        }) = message
        {
            let mesg = STORM_P2P_UNMARSHALLER.unmarshall(&*payload)?;
            self.send_app(endpoints, mesg.storm_app(), mesg.deref().clone())?;
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
        message: AppMsg,
    ) -> Result<(), DaemonError> {
        match &message {
            wrong_msg => {
                error!("Request is not supported by the APP interface");
                return Err(DaemonError::wrong_esb_msg(ServiceBus::App, wrong_msg));
            }
        }

        Ok(())
    }
}
