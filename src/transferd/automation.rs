// Storm node providing distributed storage & messaging for lightning network.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::collections::VecDeque;

use internet2::addr::NodeAddr;
use lnp_rpc::ClientId;
use storm::{ChunkId, ContainerFullId};

use super::Runtime;
use crate::bus::Endpoints;
use crate::DaemonError;

pub enum State {
    Free,
    ConnectingPeer(NodeAddr),
    AwaitingContainer {
        remote_peer: NodeAddr,
        container_id: ContainerFullId,
    },
    AwaitingChunk {
        remote_peer: NodeAddr,
        container_id: ContainerFullId,
        current: ChunkId,
        other: VecDeque<ChunkId>,
    },
}

impl Runtime {
    pub(super) fn handle_transfer(
        &mut self,
        endpoints: &mut Endpoints,
        client_id: ClientId,
        remote_peer: NodeAddr,
        container_id: ContainerFullId,
    ) -> Result<(), DaemonError> {
        // TODO: Ensure connectivity to the remote peer
        // TODO: self.send_p2p(endpoints, remote_peer, p2p::Messages::Post())?;
        // Request remote peer container data
        // Request all unknown chunks
        Ok(())
    }
}
