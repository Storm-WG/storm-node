// Storm node providing distributed storage & messaging for lightning network.
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
use internet2::presentation;
use microservices::rpc;
use microservices::util::OptionDetails;
use storm::p2p::AppMsg;
use storm::{ContainerFullId, StormApp};
use strict_encoding::{StrictDecode, StrictEncode};

use crate::FailureCode;

/// We need this wrapper type to be compatible with Storm Node having multiple message buses
#[derive(Clone, Debug, Display, From, Api)]
#[api(encoding = "strict")]
#[non_exhaustive]
pub(crate) enum BusMsg {
    #[api(type = 4)]
    #[display(inner)]
    #[from]
    Rpc(RpcMsg),

    /// Chat SUB/PUB requests
    #[api(type = 0x81)]
    #[display(inner)]
    #[from]
    Chat(RadioMsg),
}

impl rpc::Request for BusMsg {}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Display, From)]
#[derive(NetworkEncode, NetworkDecode)]
#[display(inner)]
pub enum RpcMsg {
    /* This will require LNP Node internals refactoring, in turn requiring microservice lib
    refactoring to allow multiple ids on the same controller (one per message bus).

    /// Connect to a remote peer over Bifrost protocol and ensure that the peer
    /// supports Storm protocol.
    #[display("connect{0}")]
    Connect(NodeAddr),

    /// Disconnect from a remote peer.
    Disconnect(NodeId),
     */
    /// Send a chat message to the remote peer. The peer must be connected.
    #[display("send_chat({0})")]
    SendChat(AddressedMsg<String>),

    #[display("send({0})")]
    SendContainer(AddressedMsg<AppContainer>),

    #[display("receive({0})")]
    GetContainer(AddressedMsg<AppContainer>),

    #[display("store(...)")]
    StoreFile(AddressedMsg<Vec<u8>>),

    #[display("retrieve(...)")]
    RetrieveFile(AddressedMsg<Vec<u8>>),

    // Responses to CLI
    // ----------------
    #[display("progress(\"{0}\")")]
    #[from]
    Progress(String),

    #[display("success{0}")]
    Success(OptionDetails),

    #[display("failure({0:#})")]
    #[from]
    Failure(rpc::Failure<FailureCode>),
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Display, From)]
#[derive(NetworkEncode, NetworkDecode)]
#[display(inner)]
pub enum RadioMsg {
    #[display("recv_chat({0})")]
    #[from]
    Received(AddressedMsg<String>),
}

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Debug, NetworkEncode, NetworkDecode)]
pub struct AddressedMsg<T>
where T: StrictEncode + StrictDecode
{
    pub remote_id: NodeId,
    pub data: T,
}

impl<T> Display for AddressedMsg<T>
where T: Display + StrictEncode + StrictDecode
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", self.remote_id, self.data)
    }
}

impl<T> AddressedMsg<T>
where T: StrictEncode + StrictDecode
{
    pub fn with(app_msg: AppMsg<T>, remote_peer: NodeId) -> Self {
        AddressedMsg {
            remote_id: remote_peer,
            data: app_msg.data,
        }
    }
}

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Debug, Display)]
#[derive(NetworkEncode, NetworkDecode)]
#[display("{storm_app}:{container_id}")]
pub struct AppContainer {
    pub storm_app: StormApp,
    pub container_id: ContainerFullId,
}

impl From<presentation::Error> for RpcMsg {
    fn from(err: presentation::Error) -> Self {
        RpcMsg::Failure(rpc::Failure {
            code: rpc::FailureCode::Presentation,
            info: format!("{}", err),
        })
    }
}
