// Storm node providing distributed storage & messaging for lightning network.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use internet2::session::LocalSession;
use internet2::{
    CreateUnmarshaller, SendRecvMessage, TypedEnum, Unmarshall, Unmarshaller, ZmqSocketType,
};
use lnp_rpc::ServiceId;
use microservices::error::BootstrapError;
use microservices::esb;
use microservices::esb::{EndpointList, Error};
use microservices::node::TryService;
use microservices::rpc::ClientError;
use storm_rpc::{Reply, Request};

use super::{BusMsg, ServiceBus};
use crate::{Config, DaemonError, LaunchError};

pub fn run(config: Config) -> Result<(), BootstrapError<LaunchError>> {
    // TODO: Use global context coming from LNP Node (or provide context with
    //       rust-microservices)
    let ctx = zmq::Context::new();

    let msg_endpoint = config.msg_endpoint.clone();
    let runtime = Runtime::init(config, &ctx)?;

    debug!("Connecting to LNP Node MSG service bus {}", msg_endpoint);
    let controller = esb::Controller::with(
        map! {
            ServiceBus::Msg => esb::BusConfig::with_addr(
                msg_endpoint,
                None
            )
        },
        runtime,
        ZmqSocketType::RouterConnect,
        ctx,
    )
    .map_err(|_| LaunchError::NoLnpdConnection)?;

    // TODO: This misses RPC connection; we need to mux it in
    controller.run_or_panic("stormd");

    unreachable!()
}

pub struct Runtime {
    /// Original configuration object
    pub(crate) config: Config,

    /// Stored sessions
    pub(crate) session_rpc: LocalSession,

    /// Unmarshaller instance used for parsing RPC request
    pub(crate) unmarshaller: Unmarshaller<Request>,
}

impl Runtime {
    pub fn init(config: Config, ctx: &zmq::Context) -> Result<Self, BootstrapError<LaunchError>> {
        // debug!("Initializing storage provider {:?}", config.storage_conf());
        // let storage = storage::FileDriver::with(config.storage_conf())?;

        debug!("Opening RPC API socket {}", config.rpc_endpoint);
        let session_rpc =
            LocalSession::connect(ZmqSocketType::Rep, &config.rpc_endpoint, None, None, &ctx)?;

        info!("Stormd runtime started successfully");

        Ok(Self {
            config,
            session_rpc,
            unmarshaller: Request::create_unmarshaller(),
        })
    }
}

impl TryService for Runtime {
    type ErrorType = ClientError;

    fn try_run_loop(mut self) -> Result<(), Self::ErrorType> {
        loop {
            match self.run() {
                Ok(_) => debug!("API request processing complete"),
                Err(err) => {
                    error!("Error processing API request: {}", err);
                    Err(err)?;
                }
            }
        }
    }
}

impl Runtime {
    fn run(&mut self) -> Result<(), ClientError> {
        trace!("Awaiting for ZMQ RPC requests...");
        let raw = self.session_rpc.recv_raw_message()?;
        let reply = self.rpc_process(raw).unwrap_or_else(|err| err);
        trace!("Preparing ZMQ RPC reply: {:?}", reply);
        let data = reply.serialize();
        trace!("Sending {} bytes back to the client over ZMQ RPC", data.len());
        self.session_rpc.send_raw_message(&data)?;
        Ok(())
    }
}

impl Runtime {
    pub(crate) fn rpc_process(&mut self, raw: Vec<u8>) -> Result<Reply, Reply> {
        trace!("Got {} bytes over ZMQ RPC", raw.len());
        let request = (&*self.unmarshaller.unmarshall(raw.as_slice())?).clone();
        debug!("Received ZMQ RPC request #{}: {}", request.get_type(), request);
        match request {
            Request::Noop => Ok(Reply::Success) as Result<_, DaemonError>,
        }
        .map_err(Reply::from)
    }
}

impl esb::Handler<ServiceBus> for Runtime {
    type Request = BusMsg;
    type Error = DaemonError;

    fn identity(&self) -> ServiceId { ServiceId::Storm }

    fn handle(
        &mut self,
        endpoints: &mut EndpointList<ServiceBus>,
        bus_id: ServiceBus,
        source: ServiceId,
        request: Self::Request,
    ) -> Result<(), Self::Error> {
        todo!()
    }

    fn handle_err(
        &mut self,
        endpoints: &mut EndpointList<ServiceBus>,
        error: Error<ServiceId>,
    ) -> Result<(), Self::Error> {
        todo!()
    }
}
