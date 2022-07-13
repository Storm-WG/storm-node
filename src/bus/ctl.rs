// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::fmt::{self, Display, Formatter};

use internet2::addr::NodeId;
use microservices::esb::ClientId;
use storm::p2p::AppMsg;
use storm::Container;
use storm_rpc::AppContainer;
use strict_encoding::{StrictDecode, StrictEncode};

/// RPC API requests over CTL message bus between RGB Node daemons.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display, From)]
#[derive(NetworkEncode, NetworkDecode)]
#[non_exhaustive]
pub enum CtlMsg {
    #[display("hello()")]
    Hello,

    #[display("get({0})")]
    GetContainer(AddressedClientMsg<AppContainer>),

    #[display("announce({0})")]
    AnnounceContainer(AddressedClientMsg<AppContainer>),

    #[display("send({0})")]
    SendContainer(AddressedClientMsg<AppContainer>),

    #[display("process_container(...)")]
    ProcessContainer(Container),

    #[display("processing_complete()")]
    ProcessingComplete,

    #[display("processing_failed()")]
    ProcessingFailed,
}

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Debug, NetworkEncode, NetworkDecode)]
pub struct AddressedClientMsg<T>
where T: StrictEncode + StrictDecode
{
    pub remote_id: NodeId,
    pub client_id: Option<ClientId>,
    pub data: T,
}

impl<T> Display for AddressedClientMsg<T>
where T: Display + StrictEncode + StrictDecode
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", self.remote_id, self.data)
    }
}

impl<T> AddressedClientMsg<T>
where T: StrictEncode + StrictDecode
{
    pub fn with(app_msg: AppMsg<T>, remote_peer: NodeId, client_id: ClientId) -> Self {
        AddressedClientMsg {
            client_id: Some(client_id),
            remote_id: remote_peer,
            data: app_msg.data,
        }
    }
}
