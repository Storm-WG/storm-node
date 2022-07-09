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

use crate::messages::ChatMsg;
use crate::{BusMsg, Error, RpcMsg, ServiceId};

// We have just a single service bus (RPC), so we can use any id
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default, Display)]
#[display("STORMRPC")]
struct RpcBus;

impl BusId for RpcBus {
    type Address = ServiceId;
}

type Bus = esb::EndpointList<RpcBus>;

#[repr(C)]
pub struct Client {
    client_id: ClientId,
    user_agent: String,
    response_queue: Vec<RpcMsg>,
    esb: esb::Controller<RpcBus, BusMsg, Handler>,
}

impl Client {
    pub fn with(connect: ServiceAddr, user_agent: String) -> Result<Self, Error> {
        debug!("RPC socket {}", connect);

        debug!("Setting up RPC client...");
        let client_id = rand::random();
        let bus_config = esb::BusConfig::with_addr(
            connect,
            ZmqSocketType::RouterConnect,
            Some(ServiceId::router()),
        );
        let esb = esb::Controller::with(
            map! {
                RpcBus => bus_config
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

    fn request(&mut self, req: impl Into<RpcMsg>) -> Result<(), Error> {
        let req = req.into();
        debug!("Executing {}", req);
        self.esb.send_to(RpcBus, ServiceId::Rgb, BusMsg::Rpc(req))?;
        Ok(())
    }

    fn response(&mut self) -> Result<RpcMsg, Error> {
        loop {
            if let Some(resp) = self.response_queue.pop() {
                trace!("Got response {:?}", resp);
                return Ok(resp);
            } else {
                for poll in self.esb.recv_poll()? {
                    match poll.request {
                        BusMsg::Rpc(msg) => self.response_queue.push(msg),
                    }
                }
            }
        }
    }
}

impl Client {
    pub fn chat_tell(&mut self, remote_id: NodeId, text: String) -> Result<(), Error> {
        self.request(RpcMsg::Tell(ChatMsg { remote_id, text }))
    }
}

pub struct Handler {
    identity: ServiceId,
}

// Not used in clients
impl esb::Handler<RpcBus> for Handler {
    type Request = BusMsg;
    type Error = esb::Error<ServiceId>;

    fn identity(&self) -> ServiceId { self.identity.clone() }

    fn handle(
        &mut self,
        _: &mut Bus,
        _: RpcBus,
        _: ServiceId,
        _: BusMsg,
    ) -> Result<(), Self::Error> {
        // Cli does not receive replies for now
        Ok(())
    }

    fn handle_err(&mut self, _: &mut Bus, err: esb::Error<ServiceId>) -> Result<(), Self::Error> {
        // We simply propagate the error since it already has been reported
        Err(err.into())
    }
}
