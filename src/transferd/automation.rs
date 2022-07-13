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
use std::fmt::Debug;

use internet2::addr::NodeId;
use microservices::esb::ClientId;
use storm::{p2p, ChunkId, ContainerFullId, ContainerHeader, ContainerInfo, StormApp};
use storm_rpc::DB_TABLE_CONTAINER_HEADERS;
use strict_encoding::StrictDecode;

use super::Runtime;
use crate::bus::{Endpoints, Responder};
use crate::DaemonError;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display, Error)]
#[display(doc_comments)]
pub enum AutomationError {
    /// the service is required to be in a {expected} state, while its state is {found}
    InvalidState {
        expected: StateName,
        found: StateName,
    },
}

pub type StateName = StateTy<ReceiveStateName>;
pub type State = StateTy<ReceiveState>;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display)]
#[display(Debug)]
pub enum StateTy<R>
where R: Debug
{
    Free,
    Receive(R),
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display)]
#[display(Debug)]
pub enum ReceiveStateName {
    AwaitingContainer,
    AwaitingChunk,
}

#[derive(Debug)]
pub enum ReceiveState {
    AwaitingContainer {
        client_id: Option<ClientId>,
        remote_id: NodeId,
        container_id: ContainerFullId,
    },
    AwaitingChunk {
        client_id: Option<ClientId>,
        remote_id: NodeId,
        container_id: ContainerFullId,
        current: ChunkId,
        other: VecDeque<ChunkId>,
    },
}

impl ReceiveState {
    pub fn state_name(&self) -> ReceiveStateName {
        match self {
            ReceiveState::AwaitingContainer { .. } => ReceiveStateName::AwaitingContainer,
            ReceiveState::AwaitingChunk { .. } => ReceiveStateName::AwaitingChunk,
        }
    }
}

impl State {
    pub fn state_name(&self) -> StateName {
        match self {
            State::Receive(recv) => StateName::Receive(recv.state_name()),
            State::Free => StateTy::Free,
        }
    }

    pub fn require_state(&self, expected: StateName) -> Result<(), AutomationError> {
        if self.state_name() != expected {
            Err(AutomationError::InvalidState {
                expected: StateName::Free,
                found: self.state_name(),
            })
        } else {
            Ok(())
        }
    }
}

// Receive workflow
impl Runtime {
    // TODO: Use this on receiving container announce
    pub(super) fn handle_receive(
        &mut self,
        endpoints: &mut Endpoints,
        client_id: Option<ClientId>,
        storm_app: StormApp,
        remote_id: NodeId,
        container_id: ContainerFullId,
    ) -> Result<(), DaemonError> {
        self.state.require_state(StateName::Free)?;

        // Switching the state
        self.state = State::Receive(ReceiveState::AwaitingContainer {
            client_id,
            remote_id,
            container_id,
        });

        // TODO: Ensure connectivity to the remote peer. This requires connection
        //       to LNP RPC bus

        // Request the remote peer container data
        let msg = p2p::AppMsg {
            app: storm_app,
            data: container_id,
        };
        self.send_p2p_reporting_client(
            endpoints,
            client_id,
            "Requesting container data",
            remote_id,
            p2p::Messages::PullContainer(msg),
        );

        // Request all unknown chunks
        Ok(())
    }

    pub(super) fn handle_send(
        &mut self,
        endpoints: &mut Endpoints,
        client_id: Option<ClientId>,
        storm_app: StormApp,
        remote_id: NodeId,
        id: ContainerFullId,
    ) -> Result<(), DaemonError> {
        self.state.require_state(StateName::Free)?;

        let header_chunk = self
            .store
            .retrieve_chunk(DB_TABLE_CONTAINER_HEADERS, id.container_id)?
            .ok_or(DaemonError::UnknownContainer(id.container_id))?;
        let header = ContainerHeader::strict_deserialize(header_chunk)?;
        let info = ContainerInfo { header, id };
        let msg = p2p::AppMsg {
            app: storm_app,
            data: info,
        };
        self.send_p2p_reporting_client(
            endpoints,
            client_id,
            "Sending container announcement",
            remote_id,
            p2p::Messages::AnnounceContainer(msg),
        );

        Ok(())
    }
}
