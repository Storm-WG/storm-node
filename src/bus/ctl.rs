// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use internet2::addr::NodeAddr;
use lnp_rpc::ClientId;
use storm::ContainerFullId;

/// RPC API requests over CTL message bus between RGB Node daemons.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display, From)]
#[derive(NetworkEncode, NetworkDecode)]
#[non_exhaustive]
pub enum CtlMsg {
    #[display("hello()")]
    Hello,

    #[display("send({0})")]
    Send(ContainerAddr),

    #[display("receive({0})")]
    Receive(ContainerAddr),

    #[display("processing_complete()")]
    ProcessingComplete,

    #[display("processing_failed()")]
    ProcessingFailed,
}

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Debug, Display)]
#[derive(NetworkEncode, NetworkDecode)]
#[display("{container_id}~{remote_peer}")]
pub struct ContainerAddr {
    pub client_id: ClientId,
    pub remote_peer: NodeAddr,
    pub container_id: ContainerFullId,
}
