// Storm node providing distributed storage & messaging for lightning network.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::thread;
use std::time::Duration;

use internet2::ZmqSocketType;
use microservices::error::BootstrapError;
use microservices::esb::{self, ClientId, EndpointList, Error};
use microservices::node::TryService;
use storm::Mesg;
use storm_ext::ExtMsg;
use storm_rpc::{AddressedMsg, RpcMsg, ServiceId};

use crate::bus::{BusMsg, CtlMsg, Endpoints, Responder, ServiceBus};
use crate::{Config, DaemonError, LaunchError};

pub fn run(config: Config) -> Result<(), BootstrapError<LaunchError>> {
    let rpc_endpoint = config.rpc_endpoint.clone();
    let ctl_endpoint = config.ctl_endpoint.clone();
    let ext_endpoint = config.ext_endpoint.clone();
    let chat_endpoint = config.chat_endpoint.clone();
    let runtime = Runtime::init(config)?;

    debug!(
        "Connecting to service buses {}, {}, {}, {}",
        rpc_endpoint, ctl_endpoint, ext_endpoint, chat_endpoint
    );
    let controller = esb::Controller::with(
        map! {
            ServiceBus::Storm => esb::BusConfig::with_addr(
                ext_endpoint,
                ZmqSocketType::RouterConnect,
                Some(ServiceId::stormd())
            ),
            ServiceBus::Rpc => esb::BusConfig::with_addr(
                rpc_endpoint,
                ZmqSocketType::RouterConnect,
                Some(ServiceId::stormd())
            ),
            ServiceBus::Ctl => esb::BusConfig::with_addr(
                ctl_endpoint,
                ZmqSocketType::RouterConnect,
                Some(ServiceId::stormd())
            ),
            ServiceBus::Chat => esb::BusConfig::with_subscription(
                chat_endpoint,
                ZmqSocketType::Pub,
                None
            )
        },
        runtime,
    )
    .map_err(|_| LaunchError::BusSetupFailure)?;

    controller.run_or_panic("chatd");

    unreachable!()
}

pub struct Runtime {
    #[allow(dead_code)]
    pub(super) store: store_rpc::Client,
}

impl Runtime {
    pub fn init(config: Config) -> Result<Self, BootstrapError<LaunchError>> {
        debug!("Connecting to store service at {}", config.store_endpoint);

        let store = store_rpc::Client::with(&config.store_endpoint).map_err(LaunchError::from)?;

        info!("Chat runtime started successfully");

        Ok(Self { store })
    }
}

impl Responder for Runtime {}

impl esb::Handler<ServiceBus> for Runtime {
    type Request = BusMsg;
    type Error = DaemonError;

    fn identity(&self) -> ServiceId { ServiceId::chatd() }

    fn on_ready(&mut self, endpoints: &mut EndpointList<ServiceBus>) -> Result<(), Self::Error> {
        thread::sleep(Duration::from_millis(100));
        self.send_ctl(endpoints, ServiceId::stormd(), CtlMsg::Hello)?;
        Ok(())
    }

    fn handle(
        &mut self,
        endpoints: &mut EndpointList<ServiceBus>,
        bus_id: ServiceBus,
        source: ServiceId,
        request: Self::Request,
    ) -> Result<(), Self::Error> {
        match (bus_id, request, source) {
            (ServiceBus::Storm, BusMsg::Storm(msg), service_id)
                if service_id == ServiceId::stormd() =>
            {
                self.handle_storm(endpoints, msg)
            }
            (ServiceBus::Rpc, BusMsg::Rpc(msg), ServiceId::Client(client_id)) => {
                self.handle_rpc(endpoints, client_id, msg)
            }
            (ServiceBus::Ctl, BusMsg::Ctl(msg), source) => self.handle_ctl(endpoints, source, msg),
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
    fn handle_storm(
        &mut self,
        endpoints: &mut Endpoints,
        message: ExtMsg,
    ) -> Result<(), DaemonError> {
        match message {
            ExtMsg::Post(AddressedMsg { remote_id, data }) => {
                let chat_msg = AddressedMsg {
                    remote_id,
                    data: String::from_utf8_lossy(&data.body).to_string(),
                };
                self.send_radio(endpoints, chat_msg)?;
            }
            wrong_msg => {
                error!("Request is not supported by the Storm interface");
                return Err(DaemonError::wrong_esb_msg(ServiceBus::Rpc, &wrong_msg));
            }
        }

        Ok(())
    }

    fn handle_rpc(
        &mut self,
        endpoints: &mut Endpoints,
        _client_id: ClientId,
        message: RpcMsg,
    ) -> Result<(), DaemonError> {
        match message {
            RpcMsg::SendChat(AddressedMsg { remote_id, data }) => {
                let addressed_msg = AddressedMsg {
                    remote_id,
                    data: Mesg {
                        parent_id: none!(),
                        body: data.into_bytes(),
                        container_ids: empty!(),
                    },
                };
                self.send_ext(endpoints, None, ExtMsg::Post(addressed_msg))?;
            }

            wrong_msg => {
                error!("Request is not supported by the RPC interface");
                return Err(DaemonError::wrong_esb_msg(ServiceBus::Rpc, &wrong_msg));
            }
        }

        Ok(())
    }

    fn handle_ctl(
        &mut self,
        _endpoints: &mut Endpoints,
        _source: ServiceId,
        message: CtlMsg,
    ) -> Result<(), DaemonError> {
        match message {
            wrong_msg => {
                error!("Request is not supported by the CTL interface");
                return Err(DaemonError::wrong_esb_msg(ServiceBus::Ctl, &wrong_msg));
            }
        }
    }
}
