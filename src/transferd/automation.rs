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
use std::fmt::Debug;

use internet2::addr::NodeId;
use microservices::esb::ClientId;
use storm::p2p::{ChunkPull, ChunkPush};
use storm::{
    p2p, Chunk, ChunkId, Container, ContainerFullId, ContainerHeader, ContainerId, ContainerInfo,
    StormApp,
};
use storm_rpc::{
    RpcMsg, ServiceId, DB_TABLE_CHUNKS, DB_TABLE_CONTAINERS, DB_TABLE_CONTAINER_HEADERS,
};
use strict_encoding::{StrictDecode, StrictEncode};

use super::Runtime;
use crate::bus::{CtlMsg, Endpoints, Responder};
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
    ReceivingChunks,
}

#[derive(Debug)]
pub enum ReceiveState {
    AwaitingContainer {
        info: Info,
    },
    ReceivingChunks {
        info: Info,
        total: usize,
        pending: BTreeSet<ChunkId>,
    },
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct Info {
    pub app_id: StormApp,
    pub client_id: Option<ClientId>,
    pub remote_id: NodeId,
    pub id: ContainerFullId,
}

impl ReceiveState {
    pub fn state_name(&self) -> ReceiveStateName {
        match self {
            ReceiveState::AwaitingContainer { .. } => ReceiveStateName::AwaitingContainer,
            ReceiveState::ReceivingChunks { .. } => ReceiveStateName::ReceivingChunks,
        }
    }

    pub fn info(&self) -> Info {
        match self {
            ReceiveState::AwaitingContainer { info }
            | ReceiveState::ReceivingChunks { info, .. } => *info,
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

    pub fn info(&self) -> Option<Info> {
        match self {
            State::Free => None,
            State::Receive(receive) => Some(receive.info()),
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
        id: ContainerFullId,
    ) -> Result<(), DaemonError> {
        self.state.require_state(StateName::Free)?;

        debug!("Receiving container {}", id.container_id);

        // Switching the state
        self.state = State::Receive(ReceiveState::AwaitingContainer {
            info: Info {
                app_id: storm_app,
                client_id,
                remote_id,
                id,
            },
        });

        // Request the remote peer container data
        let msg = p2p::AppMsg {
            app: storm_app,
            data: id,
        };
        self.send_p2p_reporting_client(
            endpoints,
            client_id,
            Some("Requested container"),
            remote_id,
            p2p::Messages::PullContainer(msg),
        );

        Ok(())
    }

    pub(super) fn handle_container(
        &mut self,
        endpoints: &mut Endpoints,
        container: Container,
    ) -> Result<(), DaemonError> {
        self.state.require_state(StateName::Receive(ReceiveStateName::AwaitingContainer))?;
        let info = self.state.info().expect("receive state always have metadata");

        debug!("Processing container info for {}", info.id.container_id);

        if let Some(client_id) = info.client_id {
            self.send_rpc(endpoints, client_id, RpcMsg::Progress("Container received".into()))?;
        }

        let header_chunk = Chunk::try_from(container.header.strict_serialize()?)?;
        let container_chunk = Chunk::try_from(container.strict_serialize()?)?;

        let id = container.container_id();
        self.store.store(DB_TABLE_CONTAINER_HEADERS, id, &header_chunk)?;
        self.store.store(DB_TABLE_CONTAINERS, id, &container_chunk)?;

        // Prepare list of missed chunks
        let chunk_ids = self
            .store
            .filter_unknown(DB_TABLE_CHUNKS, container.chunks.iter().copied().collect())?;
        let unknown_count = chunk_ids.len();
        if let Some(client_id) = info.client_id {
            self.send_rpc(
                endpoints,
                client_id,
                RpcMsg::Progress(format!("Retrieving {} new chunks", unknown_count).into()),
            )?;
        }

        debug!("Requesting {} chunks", chunk_ids.len());
        trace!("Requested chunk ids: {:?}", chunk_ids);

        // Switching the state
        self.state = State::Receive(ReceiveState::ReceivingChunks {
            info,
            total: unknown_count,
            pending: chunk_ids.clone(),
        });

        self.send_p2p(
            endpoints,
            info.remote_id,
            p2p::Messages::PullChunk(ChunkPull {
                app: info.app_id,
                message_id: info.id.message_id,
                container_id: info.id.container_id,
                chunk_ids: chunk_ids.clone(),
            }),
        )?;

        Ok(())
    }

    pub(super) fn handle_chunk(
        &mut self,
        endpoints: &mut Endpoints,
        chunk: Chunk,
    ) -> Result<(), DaemonError> {
        self.state.require_state(StateName::Receive(ReceiveStateName::ReceivingChunks))?;
        let info = self.state.info().expect("receive state always have metadata");

        let chunk_id = chunk.chunk_id();
        debug!("Processing chunk {}", chunk_id);

        if let Some(client_id) = info.client_id {
            self.send_rpc(
                endpoints,
                client_id,
                RpcMsg::Progress(format!("Received chunk {}", chunk_id).into()),
            )?;
        }

        self.store.store(DB_TABLE_CHUNKS, chunk_id, &chunk)?;

        // Switching the state
        match &mut self.state {
            State::Receive(ReceiveState::ReceivingChunks { pending, .. }) => {
                pending.remove(&chunk_id);
                if pending.is_empty() {
                    info!("Transfer service completed its work");
                    self.state = StateTy::Free;
                    self.send_ctl(endpoints, ServiceId::stormd(), CtlMsg::ProcessingComplete)?;
                }
            }
            _ => unreachable!(),
        }

        Ok(())
    }

    pub(super) fn handle_announce(
        &mut self,
        endpoints: &mut Endpoints,
        client_id: Option<ClientId>,
        storm_app: StormApp,
        remote_id: NodeId,
        id: ContainerFullId,
    ) -> Result<(), DaemonError> {
        self.state.require_state(StateName::Free)?;

        debug!("Sending announcement for {}", id.container_id);

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
            None,
            remote_id,
            p2p::Messages::AnnounceContainer(msg),
        );
        self.send_ctl(endpoints, ServiceId::stormd(), CtlMsg::ProcessingComplete)?;

        Ok(())
    }

    pub(super) fn handle_send_container(
        &mut self,
        endpoints: &mut Endpoints,
        client_id: Option<ClientId>,
        storm_app: StormApp,
        remote_id: NodeId,
        id: ContainerFullId,
    ) -> Result<(), DaemonError> {
        self.state.require_state(StateName::Free)?;

        debug!("Got container {}, saving to storage", id.container_id);

        let container_chunk = self
            .store
            .retrieve_chunk(DB_TABLE_CONTAINERS, id.container_id)?
            .ok_or(DaemonError::UnknownContainer(id.container_id))?;
        let container = Container::strict_deserialize(container_chunk)?;
        let msg = p2p::AppMsg {
            app: storm_app,
            data: container,
        };
        self.send_p2p_reporting_client(
            endpoints,
            client_id,
            None,
            remote_id,
            p2p::Messages::PushContainer(msg),
        );
        self.send_ctl(endpoints, ServiceId::stormd(), CtlMsg::ProcessingComplete)?;

        Ok(())
    }

    pub(super) fn handle_send_chunks(
        &mut self,
        endpoints: &mut Endpoints,
        storm_app: StormApp,
        remote_id: NodeId,
        container_id: ContainerId,
        chunk_ids: BTreeSet<ChunkId>,
    ) -> Result<(), DaemonError> {
        self.state.require_state(StateName::Free)?;

        debug!("Got request for {} chunks for {}", chunk_ids.len(), container_id);
        trace!("Requested chunks: {:?}", chunk_ids);

        for chunk_id in chunk_ids {
            // We ignore failed chunks
            if let Ok(Some(chunk)) = self.store.retrieve_chunk(DB_TABLE_CHUNKS, chunk_id) {
                let _ = self.send_p2p(
                    endpoints,
                    remote_id,
                    p2p::Messages::PushChunk(ChunkPush {
                        app: storm_app,
                        container_id,
                        chunk_id,
                        chunk,
                    }),
                );
            }
        }
        self.send_ctl(endpoints, ServiceId::stormd(), CtlMsg::ProcessingComplete)?;

        Ok(())
    }
}
