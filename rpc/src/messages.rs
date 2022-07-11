// Storm node providing distributed storage & messaging for lightning network.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use internet2::addr::NodeId;
use internet2::presentation;
use microservices::rpc;
use microservices::util::OptionDetails;
use storm::{ContainerFullId, StormApp};

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
    SendChat(ChatBulb),

    #[display("send({0})")]
    SendContainer(ContainerAddr),

    #[display("receive({0})")]
    GetContainer(ContainerAddr),

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
    Received(ChatBulb),
}

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Debug, Display)]
#[derive(NetworkEncode, NetworkDecode)]
#[display("{container_id}~{remote_id}")]
pub struct ContainerAddr {
    pub remote_id: NodeId,
    pub storm_app: StormApp,
    pub container_id: ContainerFullId,
}

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Debug, Display)]
#[derive(NetworkEncode, NetworkDecode)]
#[display("{remote_id}: {text}")]
pub struct ChatBulb {
    pub remote_id: NodeId,
    pub text: String,
}

impl From<presentation::Error> for RpcMsg {
    fn from(err: presentation::Error) -> Self {
        RpcMsg::Failure(rpc::Failure {
            code: rpc::FailureCode::Presentation,
            info: format!("{}", err),
        })
    }
}
