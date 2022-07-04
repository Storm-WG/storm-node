// Storm node providing distributed storage & messaging for lightning network.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::collections::BTreeSet;
use std::fmt::{self, Display, Formatter};

use internet2::addr::NodeId;
use microservices::rpc;
use storm::p2p::AppMsg;
use storm::{Mesg, MesgId, StormApp, Topic};
use strict_encoding::{StrictDecode, StrictEncode};

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
#[non_exhaustive]
pub enum ExtMsg {
    /// An extension app connecting to the Storm node must first signal with this message its app
    /// id. After that storm node will be able to route messages coming from Bifrost network
    /// targeting this app.
    #[api(type = 0x0100)]
    #[display("register_app({0})")]
    RegisterApp(StormApp),

    /* TODO: Consider developing sync API like
    /// Extension request to sync topics with the remote peer.
    #[api(type = 0x0004)]
    #[display("sync_topics({0})")]
    SyncTopics(NodeId),

    SyncMessages(PeerMsg<MesgFullId>),
     */
    /// List topics known to the local Storm node.
    #[api(type = 0x0102)]
    #[display("list_topics()")]
    ListTopics,

    /// Response to `ListTopics` request.
    #[api(type = 0x0103)]
    #[display("topics(...)")]
    Topics(BTreeSet<MesgId>),

    /// Sent or received propose to create a new Storm application topic which must be accepted or
    /// not.
    #[api(type = 0x0006)]
    #[display("propose_topic(...)")]
    ProposeTopic(AddressedMsg<Topic>),

    /// A message sent from Storm node to the app extension on arrival of the new information from
    /// remote peer via Bifrost network.
    #[api(type = 0x0008)]
    #[display("post_received({0})")]
    Post(AddressedMsg<Mesg>),

    /// A message from app extension to external peer requesting certain message or a topic from a
    /// remote peer.
    #[api(type = 0x000a)]
    #[display("post_retrieve({0})")]
    Read(AddressedMsg<MesgId>),

    /// Command to the storm node to decline the topic or a message with a specific id coming from
    /// certain peer.
    #[api(type = 0x000c)]
    #[display("decline({0})")]
    Decline(AddressedMsg<MesgId>),

    /// Command to the storm node to accept the topic or a message with a specific id coming from
    /// certain peer. This also requests the node to download all the unknown containers for the
    /// topic or the message.
    #[api(type = 0x000e)]
    #[display("accept({0})")]
    Accept(AddressedMsg<MesgId>),
}

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Debug, NetworkEncode, NetworkDecode)]
pub struct AddressedMsg<T>
where T: StrictEncode + StrictDecode
{
    pub remote_peer: NodeId,
    pub data: T,
}

impl<T> Display for AddressedMsg<T>
where T: Display + StrictEncode + StrictDecode
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", self.remote_peer, self.data)
    }
}

impl<T> AddressedMsg<T>
where T: StrictEncode + StrictDecode
{
    pub fn with(app_msg: AppMsg<T>, remote_peer: NodeId) -> Self {
        AddressedMsg {
            remote_peer,
            data: app_msg.data,
        }
    }
}
