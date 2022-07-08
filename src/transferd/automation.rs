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
use storm::{p2p, ChunkId, ContainerFullId, StormApp};

use super::Runtime;
use crate::bus::{Endpoints, Responder};
use crate::DaemonError;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display, Error)]
#[display(doc_comments)]
pub enum AutomationError {
    /// an incoming instruction {} requires {expected} state, while the service is in the {found}
    /// state
    InvalidState {
        instruction: Instruction,
        expected: StateName,
        found: StateName,
    },
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display, Error)]
#[display(Debug)]
pub enum Instruction {
    Transfer,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display, Error)]
#[display(Debug)]
pub enum StateName {
    Free,
    AwaitingContainer,
    AwaitingChunk,
}

#[derive(Debug)]
pub enum State {
    Free,
    AwaitingContainer {
        client_id: ClientId,
        remote_peer: NodeAddr,
        container_id: ContainerFullId,
    },
    AwaitingChunk {
        client_id: ClientId,
        remote_peer: NodeAddr,
        container_id: ContainerFullId,
        current: ChunkId,
        other: VecDeque<ChunkId>,
    },
}

impl State {
    pub fn state_name(&self) -> StateName {
        match self {
            State::Free => StateName::Free,
            State::AwaitingContainer { .. } => StateName::AwaitingContainer,
            State::AwaitingChunk { .. } => StateName::AwaitingChunk,
        }
    }

    pub fn require_state(
        &self,
        expected: StateName,
        instruction: Instruction,
    ) -> Result<(), AutomationError> {
        if self.state_name() != expected {
            Err(AutomationError::InvalidState {
                instruction,
                expected: StateName::Free,
                found: self.state_name(),
            })
        } else {
            Ok(())
        }
    }
}

impl Runtime {
    pub(super) fn handle_transfer(
        &mut self,
        endpoints: &mut Endpoints,
        client_id: ClientId,
        storm_app: StormApp,
        remote_peer: NodeAddr,
        container_id: ContainerFullId,
    ) -> Result<(), DaemonError> {
        self.state.require_state(StateName::Free, Instruction::Transfer)?;

        // Switching the state
        self.state = State::AwaitingContainer {
            client_id,
            remote_peer,
            container_id,
        };

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
            remote_peer,
            p2p::Messages::PullContainer(msg),
        );

        // Request all unknown chunks
        Ok(())
    }
}
