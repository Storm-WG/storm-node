// Storm node providing distributed storage & messaging for lightning network.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::io::{BufRead, Write};
use std::{fs, io};

use amplify::num::u24;
use amplify::IoError;
use internet2::addr::PartialNodeAddr;
use lnp::addr::LnpAddr;
use microservices::rpc::ServerError;
use storm::{Chunk, Container, ContainerHeader};
use strict_encoding::{MediumVec, StrictDecode, StrictEncode};

use crate::{Command, Opts};

#[derive(Debug, Display, Error, From)]
#[display(inner)]
pub enum Error {
    #[from]
    #[from(io::Error)]
    Io(IoError),

    #[from]
    Storm(storm_rpc::Error),

    #[from]
    Store(ServerError<store_rpc::FailureCode>),

    #[from]
    Lnp(lnp_rpc::Error),

    #[from]
    StrictEncoding(strict_encoding::Error),
}

impl Opts {
    pub fn exec(
        self,
        storm_client: &mut storm_rpc::Client,
        store_client: &mut store_rpc::Client,
        lnp_client: &mut lnp_rpc::Client,
    ) -> Result<(), Error> {
        let progress = |info| {
            println!("{}", info);
        };

        debug!("Performing {:?}", self.command);
        match self.command {
            Command::ChatSend { connect, peer } => {
                if let Some(addr) = connect {
                    let remote_node = PartialNodeAddr { id: peer, addr };
                    lnp_client.connect(LnpAddr::bifrost(remote_node))?;
                }
                let stdin = io::stdin();
                for line in stdin.lock().lines() {
                    storm_client.chat_tell(peer, line?)?;
                }
            }
            Command::ChatListen { connect, peer } => {
                if let Some(addr) = connect {
                    let remote_node = PartialNodeAddr { id: peer, addr };
                    lnp_client.connect(LnpAddr::bifrost(remote_node))?;
                }
                loop {
                    let line = storm_client.chat_recv(peer)?;
                    println!("> {}", line);
                }
            }
            Command::Containerize { mime, path, info } => {
                // TODO: Make this procedure part of Storm Core (containerization of arbitrary vec)
                let data = fs::read(path)?;
                let mut chunk_ids = MediumVec::new();
                let size = data.len() as u64;
                for piece in data.chunks(u24::MAX.into_usize()) {
                    let chunk = Chunk::try_from(piece)?;
                    let chunk_id = chunk.chunk_id();
                    store_client.store(storm_rpc::DB_TABLE_CHUNKS, chunk_id, &chunk)?;
                    chunk_ids.push(chunk_id)?;
                }

                let total_chunks = chunk_ids.len();
                let header = ContainerHeader {
                    version: 0,
                    mime,
                    info: info.unwrap_or_default(),
                    size,
                };
                let header_chunk = Chunk::try_from(header.strict_serialize()?)?;
                let container = Container {
                    header,
                    chunks: chunk_ids,
                };
                let container_chunk = Chunk::try_from(container.strict_serialize()?)?;

                let id = container.container_id();
                store_client.store(storm_rpc::DB_TABLE_CONTAINER_HEADERS, id, &header_chunk)?;
                store_client.store(storm_rpc::DB_TABLE_CONTAINERS, id, &container_chunk)?;
                eprintln!("Containerized ({} chunks in total)", total_chunks);
                println!("{}", id);
            }
            Command::Assemble { container_id, path } => {
                // TODO: Make this procedure part of Storm Core (assembling data from a container)
                let container_chunk = store_client
                    .retrieve_chunk(storm_rpc::DB_TABLE_CONTAINERS, container_id)?
                    .expect("No container with the provided id");
                let container = Container::strict_deserialize(container_chunk)?;
                println!("Size: {} bytes", container.header.size);
                println!("MIME: {}", container.header.mime);
                println!("Info: {}", container.header.info);
                let mut file = fs::File::create(&path)?;
                for chunk_id in container.chunks {
                    let chunk = store_client
                        .retrieve_chunk(storm_rpc::DB_TABLE_CHUNKS, chunk_id)?
                        .expect(&format!("Chunk {} is absent", chunk_id));
                    file.write_all(chunk.as_slice())?;
                }
                eprintln!("Saved to {}", path.display());
            }
            Command::Upload {
                connect,
                peer,
                container_id,
            } => {
                if let Some(addr) = connect {
                    let remote_node = PartialNodeAddr { id: peer, addr };
                    lnp_client.connect(LnpAddr::bifrost(remote_node))?;
                }
                storm_client.upload(peer, container_id, progress)?;
            }
            Command::Download {
                connect,
                peer,
                container_id,
            } => {
                if let Some(addr) = connect {
                    let remote_node = PartialNodeAddr { id: peer, addr };
                    lnp_client.connect(LnpAddr::bifrost(remote_node))?;
                }
                storm_client.download(peer, container_id, progress)?;
            }
        }
        Ok(())
    }
}
