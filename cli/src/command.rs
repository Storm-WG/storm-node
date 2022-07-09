// Storm node providing distributed storage & messaging for lightning network.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::io;
use std::io::BufRead;

use amplify::IoError;
use lnp::addr::LnpAddr;

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
    Lnp(lnp_rpc::Error),

    #[from]
    StrictEncoding(strict_encoding::Error),
}

impl Opts {
    pub fn exec(
        self,
        storm_client: &mut storm_rpc::Client,
        lnp_client: &mut lnp_rpc::Client,
    ) -> Result<(), Error> {
        debug!("Performing {:?}", self.command);
        match self.command {
            Command::ChatSend { peer } => {
                lnp_client.connect(LnpAddr::bifrost(peer))?;
                let stdin = io::stdin();
                for line in stdin.lock().lines() {
                    storm_client.chat_tell(peer.id, line?)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}
