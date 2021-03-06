// Storm node providing distributed storage & messaging for lightning network.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::thread::sleep;
use std::time::Duration;

use internet2::addr::{NodeId, ServiceAddr};
use internet2::ZmqSocketType;
use microservices::esb::{self, BusId, ClientId, PollItem};
use microservices::util::OptionDetails;
use storm::{ContainerFullId, ContainerId, StormApp};

use crate::messages::RadioMsg;
use crate::{AddressedMsg, AppContainer, BusMsg, Error, RpcMsg, ServiceId};

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display)]
enum Bus {
    /// RPC interface, from client to node
    #[display("RPC")]
    Rpc,

    /// Pub/sub bus used for chat daemon
    #[display("CHAT")]
    Chat,
}

impl BusId for Bus {
    type Address = ServiceId;
}

type Endpoints = esb::EndpointList<Bus>;

#[repr(C)]
pub struct Client {
    client_id: ClientId,
    user_agent: String,
    response_queue: Vec<PollItem<Bus, BusMsg>>,
    esb: esb::Controller<Bus, BusMsg, Handler>,
}

impl Client {
    pub fn with(
        rpc_endpoint: ServiceAddr,
        chat_endpoint: ServiceAddr,
        user_agent: String,
    ) -> Result<Self, Error> {
        debug!("RPC socket {}", rpc_endpoint);

        debug!("Setting up RPC client...");
        let client_id = rand::random();
        let esb = esb::Controller::with(
            map! {
                Bus::Rpc => esb::BusConfig::with_addr(
                    rpc_endpoint,
                    ZmqSocketType::RouterConnect,
                    Some(ServiceId::stormd()),
                ),
                Bus::Chat => esb::BusConfig::with_subscription(
                    chat_endpoint,
                    ZmqSocketType::Sub,
                    None,
                )
            },
            Handler {
                identity: ServiceId::Client(client_id),
            },
        )?;

        // We have to sleep in order for ZMQ to bootstrap
        sleep(Duration::from_secs_f32(0.1));

        Ok(Self {
            client_id,
            user_agent,
            response_queue: empty!(),
            esb,
        })
    }

    pub fn client_id(&self) -> ClientId { self.client_id }

    fn request(&mut self, req: impl Into<RpcMsg>, service_id: ServiceId) -> Result<(), Error> {
        let req = req.into();
        debug!("Executing {}", req);
        self.esb.send_to(Bus::Rpc, service_id, BusMsg::Rpc(req))?;
        Ok(())
    }

    fn response(&mut self) -> Result<PollItem<Bus, BusMsg>, Error> {
        loop {
            if let Some(poll) = self.response_queue.pop() {
                trace!("Got response {} from {} via {}", poll.request, poll.source, poll.bus_id);
                return Ok(poll);
            } else {
                for poll in self.esb.recv_poll()? {
                    self.response_queue.push(poll);
                }
            }
        }
    }

    fn progressive_request(
        &mut self,
        request: impl Into<RpcMsg>,
        service_id: ServiceId,
        progress: impl Fn(String),
    ) -> Result<(), Error> {
        self.request(request, service_id)?;
        loop {
            match self.response()?.request {
                BusMsg::Rpc(rpc) => match rpc.failure_to_error()? {
                    RpcMsg::Success(OptionDetails(Some(info))) => {
                        return {
                            progress(format!("Success. {}", info));
                            Ok(())
                        }
                    }
                    RpcMsg::Success(OptionDetails(None)) => {
                        return {
                            progress(s!("Success"));
                            Ok(())
                        }
                    }
                    RpcMsg::Progress(info) => progress(info),
                    _ => return Err(Error::UnexpectedServerResponse),
                },
                _ => return Err(Error::UnexpectedServerResponse),
            }
        }
    }
}

impl Client {
    pub fn chat_tell(&mut self, remote_id: NodeId, text: String) -> Result<(), Error> {
        self.request(
            RpcMsg::SendChat(AddressedMsg {
                remote_id,
                data: text,
            }),
            ServiceId::chatd(),
        )
    }

    pub fn chat_recv(&mut self, from_remote_id: NodeId) -> Result<String, Error> {
        let poll = self.response()?;
        match poll.request {
            BusMsg::Chat(RadioMsg::Received(AddressedMsg { remote_id, data }))
                if remote_id == from_remote_id =>
            {
                Ok(data)
            }
            _ => Err(Error::UnexpectedServerResponse),
        }
    }

    pub fn upload(
        &mut self,
        remote_id: NodeId,
        container_id: ContainerId,
        progress: impl Fn(String),
    ) -> Result<(), Error> {
        let msg = AddressedMsg {
            remote_id,
            data: AppContainer {
                storm_app: StormApp::FileTransfer,
                container_id: ContainerFullId {
                    // FIXME
                    message_id: zero!(),
                    container_id,
                },
            },
        };
        self.progressive_request(
            RpcMsg::SendContainer(msg),
            // TODO: Send to downpourd
            ServiceId::stormd(),
            progress,
        )
    }

    pub fn download(
        &mut self,
        remote_id: NodeId,
        container_id: ContainerId,
        progress: impl Fn(String),
    ) -> Result<(), Error> {
        let msg = AddressedMsg {
            remote_id,
            data: AppContainer {
                storm_app: StormApp::FileTransfer,
                container_id: ContainerFullId {
                    // FIXME
                    message_id: zero!(),
                    container_id,
                },
            },
        };
        self.progressive_request(
            RpcMsg::GetContainer(msg),
            // TODO: Send to downpourd
            ServiceId::stormd(),
            progress,
        )
    }
}

pub struct Handler {
    identity: ServiceId,
}

// Not used in clients
impl esb::Handler<Bus> for Handler {
    type Request = BusMsg;
    type Error = esb::Error<ServiceId>;

    fn identity(&self) -> ServiceId { self.identity.clone() }

    fn handle(
        &mut self,
        _: &mut Endpoints,
        _: Bus,
        _: ServiceId,
        _: BusMsg,
    ) -> Result<(), Self::Error> {
        // Cli does not receive replies for now
        Ok(())
    }

    fn handle_err(
        &mut self,
        _: &mut Endpoints,
        err: esb::Error<ServiceId>,
    ) -> Result<(), Self::Error> {
        // We simply propagate the error since it already has been reported
        Err(err.into())
    }
}
