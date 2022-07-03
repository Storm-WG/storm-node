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
use microservices::rpc;
use storm::{p2p, Mesg, MesgId, StormApp};

/// We need this wrapper type to be compatible with Storm Node having multiple message buses
#[derive(Clone, Debug, Display, From, Api)]
#[api(encoding = "strict")]
#[non_exhaustive]
pub(crate) enum BusMsg {
    #[api(type = 5)]
    #[display(inner)]
    #[from]
    Ext(ExtMsg),
}

impl rpc::Request for BusMsg {}

#[derive(Clone, Debug, Display, Api, From)]
#[derive(NetworkEncode, NetworkDecode)]
#[api(encoding = "strict")]
#[display(inner)]
#[non_exhaustive]
pub enum ExtMsg {
    /// An extension app connecting to the Storm node must first signal with this message its app
    /// id. After that storm node will be able to route messages coming from Bifrost network
    /// targeting this app.
    #[api(type = 0x0100)]
    RegisterApp(StormApp),

    /// A message sent from Storm node to the app extension on arrival of the new information from
    /// remote peer via Bifrost network.
    #[api(type = 0x0008)]
    Post(MesgPush),

    /// A message from app extension to external peer requesting certain message.
    #[api(type = 0x000a)]
    Read(MesgPull),

    #[api(type = 0x000c)]
    #[display("decline({0})")]
    Decline(MesgPull),

    #[api(type = 0x000e)]
    #[display("accept({0})")]
    Accept(MesgPull),
}

impl ExtMsg {
    pub fn with(p2p: p2p::Messages, remote_peer: NodeId) -> Option<Self> {
        match p2p {
            p2p::Messages::Post(req) => Some(ExtMsg::Post(MesgPush {
                remote_peer,
                message: req.message,
            })),
            p2p::Messages::Read(req) => Some(ExtMsg::Read(MesgPull {
                remote_peer,
                message_id: req.message_id,
            })),
            p2p::Messages::Decline(req) => Some(ExtMsg::Decline(MesgPull {
                remote_peer,
                message_id: req.message_id,
            })),
            p2p::Messages::Accept(req) => Some(ExtMsg::Accept(MesgPull {
                remote_peer,
                message_id: req.message_id,
            })),

            _ => None,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Display, NetworkEncode, NetworkDecode)]
#[display("push({remote_peer}, {message})")]
pub struct MesgPush {
    pub remote_peer: NodeId,
    pub message: Mesg,
}

#[derive(Clone, PartialEq, Eq, Debug, Display, NetworkEncode, NetworkDecode)]
#[display("pull({remote_peer}, {message_id})")]
pub struct MesgPull {
    pub remote_peer: NodeId,
    pub message_id: MesgId,
}
