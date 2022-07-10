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
use microservices::esb::{self, BusId, ClientId};
use storm::StormApp;

use crate::messages::ChatMsg;
use crate::{BusMsg, Error, RpcMsg, ServiceId};

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
    response_queue: Vec<(RpcMsg, ServiceId)>,
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
                Bus::Chat => esb::BusConfig::with_addr(
                    chat_endpoint,
                    ZmqSocketType::Sub,
                    Some(ServiceId::stormd()),
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

    fn response(&mut self) -> Result<(RpcMsg, ServiceId), Error> {
        loop {
            if let Some((resp, service_id)) = self.response_queue.pop() {
                trace!("Got response {:?} from {}", resp, service_id);
                return Ok((resp, service_id));
            } else {
                for poll in self.esb.recv_poll()? {
                    match poll.request {
                        BusMsg::Rpc(msg) => self.response_queue.push((msg, poll.source)),
                    }
                }
            }
        }
    }
}

impl Client {
    pub fn chat_tell(&mut self, remote_id: NodeId, text: String) -> Result<(), Error> {
        self.request(RpcMsg::SendChatMsg(ChatMsg { remote_id, text }), ServiceId::chatd())
    }

    pub fn chat_recv(&mut self, from_remote_id: NodeId) -> Result<Result<String, RpcMsg>, Error> {
        match self.response()? {
            (
                RpcMsg::ReceivedChatMsg(ChatMsg { remote_id, text }),
                ServiceId::StormApp(StormApp::Chat),
            ) if remote_id == from_remote_id => Ok(Ok(text)),
            (msg, _) => Ok(Err(msg)),
        }
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
