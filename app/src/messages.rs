// Storm node providing distributed storage & messaging for lightning network.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use internet2::presentation;
use microservices::rpc;
use storm::{p2p, Chunk, ChunkId, Container, Mesg, MesgId, StormApp};
use storm_rpc::FailureCode;

/// We need this wrapper type to be compatible with Storm Node having multiple message buses
#[derive(Clone, Debug, Display, From, Api)]
#[api(encoding = "strict")]
#[non_exhaustive]
pub(crate) enum BusMsg {
    #[api(type = 5)]
    #[display(inner)]
    #[from]
    App(AppMsg),
}

impl rpc::Request for BusMsg {}

#[derive(Clone, Debug, Display, Api, From)]
#[derive(NetworkEncode, NetworkDecode)]
#[api(encoding = "strict")]
#[display(inner)]
#[non_exhaustive]
pub enum AppMsg {
    #[api(type = 0x0100)]
    RegisterApp(StormApp),

    #[api(type = 0x0010)]
    Post(MesgPush),

    #[api(type = 0x0011)]
    Read(MesgPull),

    #[api(type = 0x0012)]
    Push(Chunk),

    #[api(type = 0x0013)]
    Chunk(ChunkId),

    #[api(type = 0x0020)]
    Decline(MesgId),

    // Responses to CLI
    // ----------------
    #[display("success({0})")]
    #[api(type = 0x0001)]
    Success,

    #[display("failure({0:#})")]
    #[api(type = 0x0000)]
    #[from]
    Failure(rpc::Failure<FailureCode>),
}

impl From<presentation::Error> for AppMsg {
    fn from(err: presentation::Error) -> Self {
        AppMsg::Failure(rpc::Failure {
            code: rpc::FailureCode::Presentation,
            info: format!("{}", err),
        })
    }
}

impl From<storm::p2p::Messages> for AppMsg {
    fn from(p2p: p2p::Messages) -> Self {
        match p2p {
            p2p::Messages::Post(p2p::PostReq {
                message, container, ..
            }) => AppMsg::Post(MesgPush { message, container }),
            p2p::Messages::Read(p2p::ReadReq {
                message_id,
                with_container,
                ..
            }) => AppMsg::Read(MesgPull {
                message_id,
                with_container,
            }),
            p2p::Messages::Push(p2p::ChunkPush { chunk, .. }) => AppMsg::Push(chunk),
            p2p::Messages::Chunk(p2p::ChunkPull { chunk_id, .. }) => AppMsg::Chunk(chunk_id),
            p2p::Messages::Decline(p2p::DeclineResp { mesg_id, .. }) => AppMsg::Decline(mesg_id),
            _ => unreachable!("Storm node uses outdated application API"),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Display, NetworkEncode, NetworkDecode)]
#[display("post({message}, ...)")]
pub struct MesgPush {
    pub message: Mesg,
    pub container: Option<Container>,
}

#[derive(Clone, PartialEq, Eq, Debug, Display, NetworkEncode, NetworkDecode)]
#[display("read({message_id}, {with_container})")]
pub struct MesgPull {
    pub message_id: MesgId,
    pub with_container: bool,
}
