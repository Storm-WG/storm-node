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

use internet2::addr::NodeId;
use microservices::rpc;
use storm::p2p::{self, AppMsg};
use storm::{ContainerFullId, ContainerInfo, Mesg, MesgId, StormApp, Topic};
use storm_rpc::AddressedMsg;
use strict_encoding::StrictEncode;

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
    ListTopics(AddressedMsg<()>),

    /// Response to `ListTopics` request.
    #[api(type = 0x0103)]
    #[display("topics(...)")]
    Topics(AddressedMsg<BTreeSet<MesgId>>),

    /// Sent or received propose to create a new Storm application topic which must be accepted or
    /// not.
    #[api(type = 0x0006)]
    #[display("propose_topic(...)")]
    ProposeTopic(AddressedMsg<Topic>),

    /// A message sent from Storm node to the app extension on arrival of the new information from
    /// remote peer via Bifrost network and from the app extension to the Storm node on sent.
    #[api(type = 0x0008)]
    #[display("post({0})")]
    Post(AddressedMsg<Mesg>),

    /// A message from app extension to external peer requesting certain message or a topic from a
    /// remote peer.
    #[api(type = 0x000a)]
    #[display("post_retrieve({0})")]
    Read(AddressedMsg<MesgId>),

    /// A received container announcement
    #[api(type = 0x0011)]
    #[display("container_announcement({0})")]
    ContainerAnnouncement(AddressedMsg<ContainerInfo>),

    /// Command from an extension to the main daemon to retrieve container from the remote peer
    #[api(type = 0x0012)]
    #[display("retrieve_container({0})")]
    RetrieveContainer(AddressedMsg<ContainerFullId>),

    /// Command from an extension to the main daemon to send container to the remote peer
    #[api(type = 0x0014)]
    #[display("send_container({0})")]
    SendContainer(AddressedMsg<ContainerFullId>),

    /// Command to the storm node to decline the topic or a message with a specific id coming from
    /// certain peer.
    #[api(type = 0x001c)]
    #[display("decline({0})")]
    Decline(AddressedMsg<MesgId>),

    /// Command to the storm node to accept the topic or a message with a specific id coming from
    /// certain peer. This also requests the node to download all the unknown containers for the
    /// topic or the message.
    #[api(type = 0x001e)]
    #[display("accept({0})")]
    Accept(AddressedMsg<MesgId>),
}

pub trait StormExtMsg {
    fn storm_ext_msg(self, remote_id: NodeId) -> Result<(StormApp, ExtMsg), Self>
    where Self: Sized;
}

impl StormExtMsg for p2p::Messages {
    fn storm_ext_msg(self, remote_id: NodeId) -> Result<(StormApp, ExtMsg), Self> {
        Ok(match self {
            p2p::Messages::ListTopics(AppMsg { data, app }) => {
                (app, ExtMsg::ListTopics(AddressedMsg { remote_id, data }))
            }
            p2p::Messages::AppTopics(AppMsg { data, app }) => {
                (app, ExtMsg::Topics(AddressedMsg { remote_id, data }))
            }
            p2p::Messages::ProposeTopic(AppMsg { data, app }) => {
                (app, ExtMsg::ProposeTopic(AddressedMsg { remote_id, data }))
            }
            p2p::Messages::Post(AppMsg { data, app }) => {
                (app, ExtMsg::Post(AddressedMsg { remote_id, data }))
            }
            p2p::Messages::Read(AppMsg { data, app }) => {
                (app, ExtMsg::Read(AddressedMsg { remote_id, data }))
            }
            p2p::Messages::AnnounceContainer(AppMsg { data, app }) => {
                (app, ExtMsg::ContainerAnnouncement(AddressedMsg { remote_id, data }))
            }
            p2p::Messages::PullContainer(AppMsg { data, app }) => {
                (app, ExtMsg::RetrieveContainer(AddressedMsg { remote_id, data }))
            }
            p2p::Messages::Decline(AppMsg { data, app }) => {
                (app, ExtMsg::Decline(AddressedMsg { remote_id, data }))
            }
            p2p::Messages::Accept(AppMsg { data, app }) => {
                (app, ExtMsg::Accept(AddressedMsg { remote_id, data }))
            }
            other => return Err(other),
        })
    }
}

impl ExtMsg {
    pub fn remote_id(&self) -> NodeId {
        match self {
            ExtMsg::RegisterApp(_) => {
                unreachable!("ExtMsg::remote_id must not be called on ExtMsg::RegisterApp")
            }
            ExtMsg::ListTopics(AddressedMsg { remote_id, .. })
            | ExtMsg::Topics(AddressedMsg { remote_id, .. })
            | ExtMsg::ProposeTopic(AddressedMsg { remote_id, .. })
            | ExtMsg::Post(AddressedMsg { remote_id, .. })
            | ExtMsg::Read(AddressedMsg { remote_id, .. })
            | ExtMsg::ContainerAnnouncement(AddressedMsg { remote_id, .. })
            | ExtMsg::RetrieveContainer(AddressedMsg { remote_id, .. })
            | ExtMsg::SendContainer(AddressedMsg { remote_id, .. })
            | ExtMsg::Decline(AddressedMsg { remote_id, .. })
            | ExtMsg::Accept(AddressedMsg { remote_id, .. }) => *remote_id,
        }
    }

    pub fn p2p_message(self, app: StormApp) -> p2p::Messages {
        match self {
            ExtMsg::RegisterApp(_) => {
                unreachable!("ExtMsg::remote_id must not be called on ExtMsg::RegisterApp")
            }
            ExtMsg::ListTopics(AddressedMsg { data, .. }) => {
                p2p::Messages::ListTopics(AppMsg { app, data })
            }
            ExtMsg::Topics(AddressedMsg { data, .. }) => {
                p2p::Messages::AppTopics(AppMsg { app, data })
            }
            ExtMsg::ProposeTopic(AddressedMsg { data, .. }) => {
                p2p::Messages::ProposeTopic(AppMsg { app, data })
            }
            ExtMsg::Post(AddressedMsg { data, .. }) => p2p::Messages::Post(AppMsg { app, data }),
            ExtMsg::Read(AddressedMsg { data, .. }) => p2p::Messages::Read(AppMsg { app, data }),
            ExtMsg::Decline(AddressedMsg { data, .. }) => {
                p2p::Messages::Decline(AppMsg { app, data })
            }
            ExtMsg::Accept(AddressedMsg { data, .. }) => {
                p2p::Messages::Accept(AppMsg { app, data })
            }
            ExtMsg::ContainerAnnouncement(AddressedMsg { data, .. }) => {
                p2p::Messages::AnnounceContainer(AppMsg { app, data })
            }
            ExtMsg::SendContainer(AddressedMsg { .. }) => {
                unreachable!("the task is handled by a dedicated daemon")
            }
            ExtMsg::RetrieveContainer(AddressedMsg { .. }) => {
                unreachable!("the task is handled by a dedicated daemon")
            }
        }
    }

    pub fn to_payload(&self) -> Vec<u8> {
        match self {
            ExtMsg::RegisterApp(_) => {
                unreachable!("ExtMsg::remote_id must not be called on ExtMsg::RegisterApp")
            }
            ExtMsg::ListTopics(AddressedMsg { data, .. }) => data.strict_serialize(),
            ExtMsg::Topics(AddressedMsg { data, .. }) => data.strict_serialize(),
            ExtMsg::ProposeTopic(AddressedMsg { data, .. }) => data.strict_serialize(),
            ExtMsg::Post(AddressedMsg { data, .. }) => data.strict_serialize(),
            ExtMsg::Read(AddressedMsg { data, .. }) => data.strict_serialize(),
            ExtMsg::Decline(AddressedMsg { data, .. }) => data.strict_serialize(),
            ExtMsg::Accept(AddressedMsg { data, .. }) => data.strict_serialize(),
            ExtMsg::ContainerAnnouncement(AddressedMsg { data, .. }) => data.strict_serialize(),
            ExtMsg::SendContainer(AddressedMsg { data, .. }) => data.strict_serialize(),
            ExtMsg::RetrieveContainer(AddressedMsg { data, .. }) => data.strict_serialize(),
        }
        .expect("extension-generated message can't be serialized as a bifrost message payload")
    }
}
