// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::str::FromStr;

use internet2::addr::NodeId;
use lnp2p::bifrost::BifrostApp;
use microservices::esb;
use microservices::esb::{ClientId, ServiceName};
use storm::StormApp;
use strict_encoding::{strict_deserialize, strict_serialize};

pub type DaemonId = u64;

/// Identifiers of daemons participating in LNP Node
#[derive(Clone, PartialEq, Eq, Hash, Debug, Display, From, StrictEncode, StrictDecode)]
pub enum ServiceId {
    #[display("stormd")]
    #[strict_encoding(value = 0x24)] // This mimics Bifrost LNP storm service id
    MsgApp(BifrostApp),

    #[display("chapp<{0}>")]
    #[strict_encoding(value = 0x24)]
    ChannelApp(BifrostApp),

    #[display("app<{0}>")]
    #[strict_encoding(value = 0x41)]
    StormApp(StormApp),

    #[display("client<{0}>")]
    #[strict_encoding(value = 2)]
    Client(ClientId),

    #[display("lnpd")]
    #[strict_encoding(value = 0x20)]
    Lnp,

    #[display("peerd<biffrost, {0}>")]
    #[from]
    #[strict_encoding(value = 0x22)]
    Peer(NodeId),

    #[display("tansferd<{0}>")]
    #[strict_encoding(value = 0x42)]
    Transfer(DaemonId),

    #[display("other<{0}>")]
    #[strict_encoding(value = 0xFF)]
    Other(ServiceName),
}

impl ServiceId {
    pub fn stormd() -> ServiceId { ServiceId::MsgApp(BifrostApp::Storm) }
    pub fn chatd() -> ServiceId { ServiceId::StormApp(StormApp::Chat) }
}

impl esb::ServiceAddress for ServiceId {}

impl From<ServiceId> for Vec<u8> {
    fn from(daemon_id: ServiceId) -> Self {
        strict_serialize(&daemon_id).expect("Memory-based encoding does not fail")
    }
}

impl From<Vec<u8>> for ServiceId {
    fn from(vec: Vec<u8>) -> Self {
        strict_deserialize(&vec).unwrap_or_else(|_| {
            ServiceId::Other(
                ServiceName::from_str(&String::from_utf8_lossy(&vec))
                    .expect("ClientName conversion never fails"),
            )
        })
    }
}
