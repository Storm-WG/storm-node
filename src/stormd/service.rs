// Storm node providing distributed storage & messaging for lightning network.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use internet2::{CreateUnmarshaller, Unmarshaller, ZmqSocketType};
use lnp_rpc::ServiceId;
use microservices::error::BootstrapError;
use microservices::esb;
use microservices::esb::{EndpointList, Error};
use microservices::node::TryService;
use storm_rpc::RpcMsg;

use super::{BusMsg, ServiceBus};
use crate::{Config, DaemonError, LaunchError};

pub fn run(config: Config) -> Result<(), BootstrapError<LaunchError>> {
    let msg_endpoint = config.msg_endpoint.clone();
    let rpc_endpoint = config.rpc_endpoint.clone();
    let runtime = Runtime::init(config)?;

    debug!("Connecting to service bus {}", msg_endpoint);
    let controller = esb::Controller::with(
        map! {
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
