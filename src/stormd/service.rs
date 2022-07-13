// Storm node providing distributed storage & messaging for lightning network.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use std::ops::Deref;

use internet2::addr::NodeId;
use internet2::{Unmarshall, ZmqSocketType};
use lnp2p::bifrost;
use lnp2p::bifrost::{BifrostApp, Messages as LnMsg};
use microservices::cli::LogStyle;
use microservices::error::BootstrapError;
use microservices::esb;
use microservices::esb::{ClientId, EndpointList, Error};
use microservices::node::TryService;
use storm::p2p::{AppMsg, ChunkPull, ChunkPush, Messages, STORM_P2P_UNMARSHALLER};
use storm::{ContainerId, StormApp};
use storm_ext::{ExtMsg, StormExtMsg};
use storm_rpc::{
    AddressedMsg, AppContainer, RpcMsg, ServiceId, DB_TABLE_CHUNKS, DB_TABLE_CONTAINERS,
    DB_TABLE_CONTAINER_HEADERS,
};

use crate::bus::{
    AddressedClientMsg, BusMsg, ChunkSend, CtlMsg, DaemonId, Endpoints, Responder, ServiceBus,
};
use crate::stormd::Daemon;
use crate::{Config, DaemonError, LaunchError};

pub fn run(config: Config<super::Config>) -> Result<(), BootstrapError<LaunchError>> {
    let msg_endpoint = config.msg_endpoint.clone();
    let rpc_endpoint = config.rpc_endpoint.clone();
    let ctl_endpoint = config.ctl_endpoint.clone();
    let ext_endpoint = config.ext_endpoint.clone();
    let runtime = Runtime::init(config)?;

    debug!("Connecting to service bus {}", msg_endpoint);
    let controller = esb::Controller::with(
        map! {
            ServiceBus::Storm => esb::BusConfig::with_addr(
                ext_endpoint,
                ZmqSocketType::RouterBind,
                None,
            ),
            ServiceBus::Ctl => esb::BusConfig::with_addr(
                ctl_endpoint,
                ZmqSocketType::RouterBind,
                None,
            ),
            ServiceBus::Msg => esb::BusConfig::with_addr(
                msg_endpoint,
                ZmqSocketType::RouterConnect,
                Some(ServiceId::Lnp)
            ),
            ServiceBus::Rpc => esb::BusConfig::with_addr(
                rpc_endpoint,
                ZmqSocketType::RouterBind,
                None
            )
        },
        runtime,
    )
    .map_err(|_| LaunchError::BusSetupFailure)?;

    controller.run_or_panic("stormd");

    unreachable!()
}

pub struct Runtime {
    pub(super) config: Config<super::Config>,
    pub(super) registered_apps: BTreeSet<StormApp>,

    pub(crate) store: store_rpc::Client,

    pub(crate) transferd_free: VecDeque<DaemonId>,
    pub(crate) transferd_busy: HashSet<DaemonId>,
    pub(crate) container_transfers: HashMap<ContainerId, DaemonId>,
    pub(crate) ctl_queue: VecDeque<CtlMsg>,
}

impl Runtime {
    pub fn init(config: Config<super::Config>) -> Result<Self, BootstrapError<LaunchError>> {
        debug!("Connecting to store service at {}", config.store_endpoint);

        let mut store =
            store_rpc::Client::with(&config.store_endpoint).map_err(LaunchError::from)?;

        for table in [DB_TABLE_CONTAINER_HEADERS, DB_TABLE_CONTAINERS, DB_TABLE_CHUNKS] {
            store.use_table(table.to_owned()).map_err(LaunchError::from)?;
        }

        info!("Stormd runtime started successfully");

        Ok(Self {
            config,
            store,
            registered_apps: empty!(),
            transferd_free: empty!(),
            transferd_busy: empty!(),
            container_transfers: empty!(),
            ctl_queue: empty!(),
        })
    }
}

impl Responder for Runtime {}

impl esb::Handler<ServiceBus> for Runtime {
    type Request = BusMsg;
    type Error = DaemonError;

    fn identity(&self) -> ServiceId { ServiceId::stormd() }

    fn on_ready(&mut self, _senders: &mut Endpoints) -> Result<(), Self::Error> {
        if self.config.ext.run_chat {
            info!("Starting chat daemon...");
            let config = Config::with(self.config.clone(), ());
            self.launch_daemon(Daemon::Chatd, config)?;
        }
        if self.config.ext.run_downpour {
            info!("Starting downpour daemon...");
            let config = Config::with(self.config.clone(), ());
            self.launch_daemon(Daemon::Downpourd, config)?;
        }
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
            (ServiceBus::Msg, BusMsg::Bifrost(msg), ServiceId::Peer(remote_id)) => {
                self.handle_p2p(endpoints, remote_id, msg)
            }
            (ServiceBus::Ctl, BusMsg::Ctl(msg), source) => self.handle_ctl(endpoints, source, msg),
            (ServiceBus::Storm, BusMsg::Storm(msg), ServiceId::StormApp(app_id)) => {
                self.handle_app(endpoints, app_id, msg)
            }
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
        remote_id: NodeId,
        message: LnMsg,
    ) -> Result<(), DaemonError> {
        if let LnMsg::Message(bifrost::Msg {
            app: BifrostApp::Storm,
            payload,
        }) = &message
        {
            let mesg = STORM_P2P_UNMARSHALLER.unmarshall(&**payload)?.deref().clone();

            /* Messages::PullContainer(_) => {} */
            // Messages::Reject(_) => {}
            // Messages::PullChunk(_) => {}
            // Messages::PushChunk(_) => {}
            if matches!(
                mesg,
                Messages::PushContainer(_) | Messages::PullChunk(_) | Messages::PushChunk(_)
            ) {
                debug!("Processing container transfer request {}", mesg);

                // TODO: Ensure that the incoming chunks references correct app id and message id
                let (container_id, instr) = match mesg {
                    // These should be processed by transfer service
                    Messages::PushContainer(AppMsg { app, data }) => {
                        (data.container_id(), CtlMsg::ProcessContainer(data))
                    }
                    Messages::PullChunk(ChunkPull {
                        app,
                        message_id,
                        container_id,
                        chunk_ids,
                    }) => (
                        container_id,
                        CtlMsg::SendChunks(AddressedMsg {
                            remote_id,
                            data: ChunkSend {
                                storm_app: app,
                                container_id,
                                chunk_ids,
                            },
                        }),
                    ),
                    Messages::PushChunk(ChunkPush {
                        app,
                        container_id,
                        chunk_id,
                        chunk,
                    }) => (container_id, CtlMsg::ProcessChunk(chunk)),
                    _ => unreachable!(),
                };

                if let Some(daemon_id) = self.container_transfers.get(&container_id) {
                    self.send_ctl(endpoints, ServiceId::Transfer(*daemon_id), instr)?;
                } else if matches!(instr, CtlMsg::SendChunks(_)) {
                    self.ctl_queue.push_back(instr);
                    self.pick_or_start(endpoints, None)?;
                } else {
                    warn!("No active transfer is known for requested {}", container_id);
                };

                return Ok(());
            }

            match mesg.storm_ext_msg(remote_id) {
                Ok((app, storm_msg)) => self.send_ext(endpoints, Some(app), storm_msg)?,

                // Messages we process ourselves
                Err(Messages::ListApps) => {
                    self.send_p2p(
                        endpoints,
                        remote_id,
                        Messages::ActiveApps(self.registered_apps.clone()),
                    )?;
                }

                // A remote peer described list of apps. We need to report that to a client.
                Err(Messages::ActiveApps(_)) => {}

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
            RpcMsg::SendContainer(container) => {
                self.ctl_queue.push_back(CtlMsg::AnnounceContainer(AddressedClientMsg {
                    remote_id: container.remote_id,
                    client_id: Some(client_id),
                    data: container.data,
                }));
                self.pick_or_start(endpoints, Some(client_id))
            }

            RpcMsg::GetContainer(container) => {
                self.ctl_queue.push_back(CtlMsg::GetContainer(AddressedClientMsg {
                    remote_id: container.remote_id,
                    client_id: Some(client_id),
                    data: container.data,
                }));
                self.pick_or_start(endpoints, Some(client_id))
            }

            wrong_msg => {
                error!("Request is not supported by the RPC interface");
                Err(DaemonError::wrong_esb_msg(ServiceBus::Rpc, &wrong_msg))
            }
        }
    }

    fn handle_ctl(
        &mut self,
        endpoints: &mut Endpoints,
        source: ServiceId,
        message: CtlMsg,
    ) -> Result<(), DaemonError> {
        match &message {
            CtlMsg::Hello => {
                if matches!(source, ServiceId::Transfer(_)) {
                    self.accept_daemon(source)?;
                    self.pick_task(endpoints)?;
                }
                // TODO: Register other daemons
            }

            CtlMsg::ProcessingFailed | CtlMsg::ProcessingComplete => {
                if let ServiceId::Transfer(daemon_id) = source {
                    if let Some(pos) = self
                        .container_transfers
                        .iter()
                        .find(|(_, id)| daemon_id == **id)
                        .map(|(a, _)| a)
                        .copied()
                    {
                        self.container_transfers.remove(&pos);
                    }
                    self.transferd_busy.remove(&daemon_id);
                    self.transferd_free.push_back(daemon_id);
                    self.pick_task(endpoints)?;
                }
            }

            wrong_msg => {
                error!("Request is not supported by the CTL interface");
                return Err(DaemonError::wrong_esb_msg(ServiceBus::Ctl, wrong_msg));
            }
        }

        Ok(())
    }

    fn handle_app(
        &mut self,
        endpoints: &mut Endpoints,
        app: StormApp,
        message: ExtMsg,
    ) -> Result<(), DaemonError> {
        match message {
            ExtMsg::RegisterApp(app_id) => {
                if app == app_id {
                    info!("Application {} is registered", app_id);
                    self.registered_apps.insert(app_id);
                } else {
                    error!(
                        "Request on application {} registration issued by a non-application \
                         daemon {}",
                        app,
                        ServiceId::StormApp(app_id)
                    );
                    return Err(DaemonError::wrong_esb_msg_source(
                        ServiceBus::Storm,
                        &message,
                        ServiceId::StormApp(app_id),
                    ));
                }
            }

            ExtMsg::RetrieveContainer(container) => {
                self.ctl_queue.push_back(CtlMsg::GetContainer(AddressedClientMsg {
                    remote_id: container.remote_id,
                    client_id: None,
                    data: AppContainer {
                        storm_app: app,
                        container_id: container.data,
                    },
                }));
                self.pick_or_start(endpoints, None)?;
            }

            ExtMsg::SendContainer(container) => {
                self.ctl_queue.push_back(CtlMsg::SendContainer(AddressedClientMsg {
                    remote_id: container.remote_id,
                    client_id: None,
                    data: AppContainer {
                        storm_app: app,
                        container_id: container.data,
                    },
                }));
                self.pick_or_start(endpoints, None)?;
            }

            // We need to the rest of the messages to the Bifrost network
            forward => {
                self.send_p2p(endpoints, forward.remote_id(), forward.p2p_message(app))?;
            }
        }

        Ok(())
    }
}

impl Runtime {
    fn accept_daemon(&mut self, source: ServiceId) -> Result<(), esb::Error<ServiceId>> {
        info!("{} daemon is {}", source.ended(), "connected".ended());

        match source {
            service_id if service_id == ServiceId::stormd() => {
                error!("{}", "Unexpected another Stormd instance connection".err());
            }
            ServiceId::Transfer(daemon_id) => {
                self.transferd_free.push_back(daemon_id);
                info!(
                    "Transfer service {} is registered; total {} container processors are known",
                    daemon_id,
                    self.transferd_free.len() + self.transferd_busy.len()
                );
            }
            _ => {
                // Ignoring the rest of daemon/client types
            }
        }

        Ok(())
    }

    fn pick_task(&mut self, endpoints: &mut Endpoints) -> Result<bool, esb::Error<ServiceId>> {
        if self.ctl_queue.is_empty() {
            return Ok(true);
        }

        let (service, daemon_id) = match self.transferd_free.front() {
            Some(damon_id) => (ServiceId::Transfer(*damon_id), *damon_id),
            None => return Ok(false),
        };

        let msg = match self.ctl_queue.pop_front() {
            None => return Ok(true),
            Some(req) => req,
        };

        debug!("Assigning task {} to {}", msg, service);

        let container_id = match msg {
            CtlMsg::GetContainer(AddressedClientMsg {
                data: AppContainer { container_id, .. },
                ..
            })
            | CtlMsg::SendContainer(AddressedClientMsg {
                data: AppContainer { container_id, .. },
                ..
            }) => Some(container_id.container_id),
            _ => None,
        };
        self.send_ctl(endpoints, service, msg)?;

        if let Some(container_id) = container_id {
            self.container_transfers.insert(container_id, daemon_id);
        }
        self.transferd_free.pop_front();
        self.transferd_busy.insert(daemon_id);

        Ok(true)
    }

    fn pick_or_start(
        &mut self,
        endpoints: &mut Endpoints,
        client_id: Option<ClientId>,
    ) -> Result<(), DaemonError> {
        if self.pick_task(endpoints)? {
            if let Some(client_id) = client_id {
                let _ = self.send_rpc(
                    endpoints,
                    client_id,
                    RpcMsg::Progress(s!("Container send request is forwarded to transfer service")),
                );
            }
            return Ok(());
        }

        let config = self.config.clone().into();
        let _handle = self.launch_daemon(Daemon::Transferd, config)?;
        if let Some(client_id) = client_id {
            let _ = self.send_rpc(
                endpoints,
                client_id,
                RpcMsg::Progress(s!("A new transfer service instance is started")),
            );
        }

        // TODO: Store daemon handlers
        Ok(())
    }
}
