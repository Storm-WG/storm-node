// Storm node providing distributed storage & messaging for lightning network.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use microservices::rpc::ServerError;
use microservices::shell::Exec;
use storm_rpc::{Client, FailureCode};

use crate::{Command, Opts};

impl Opts {
    pub fn exec(
        self,
        storm_client: &mut storm_rpc::Client,
        lnp_client: &mut lnp_rpc::Client,
    ) -> Result<(), ServerError<FailureCode>> {
        debug!("Performing {:?}", self.command);
        match self.command {
            _ => {}
        }
        Ok(())
    }
}
