// Storm node providing distributed storage & messaging for lightning network.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use internet2::addr::ServiceAddr;
use internet2::session::LocalSession;
use internet2::{
    CreateUnmarshaller, SendRecvMessage, TypedEnum, Unmarshall, Unmarshaller, ZmqSocketType,
};
use microservices::rpc::ServerError;
use microservices::ZMQ_CONTEXT;

use crate::messages::BusMsg;
use crate::{FailureCode, RpcMsg};

pub struct Client {
    // TODO: Replace with RpcSession once its implementation is completed
    session_rpc: LocalSession,
    unmarshaller: Unmarshaller<BusMsg>,
}

impl Client {
    pub fn with(connect: &ServiceAddr) -> Result<Self, ServerError<FailureCode>> {
        // TODO: Connect to ESB bus instead
        debug!("Initializing runtime");
        trace!("Connecting to storm daemon at {}", connect);
        let session_rpc =
            LocalSession::connect(ZmqSocketType::RouterConnect, connect, None, None, &ZMQ_CONTEXT)?;
        Ok(Self {
            session_rpc,
            unmarshaller: BusMsg::create_unmarshaller(),
        })
    }

    pub fn request(&mut self, request: RpcMsg) -> Result<RpcMsg, ServerError<FailureCode>> {
        trace!("Sending request to the server: {:?}", request);
        let data = BusMsg::from(request).serialize();
        trace!("Raw request data ({} bytes): {:02X?}", data.len(), data);
        self.session_rpc.send_raw_message(&data)?;
        trace!("Awaiting reply");
        let raw = self.session_rpc.recv_raw_message()?;
        trace!("Got reply ({} bytes), parsing: {:02X?}", raw.len(), raw);
        let reply = self.unmarshaller.unmarshall(raw.as_slice())?;
        trace!("Reply: {:?}", reply);
        match &*reply {
            BusMsg::Rpc(rpc) => Ok(rpc.clone()),
        }
    }
}
