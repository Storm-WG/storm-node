// Storm node providing distributed storage & messaging for lightning network.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use super::Opts;
use crate::opts::Options;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct Config {
    pub run_chat: bool,
    pub run_downpour: bool,
    /// Indicates whether deamons should be spawned as threads (true) or as child processes (false)
    pub threaded: bool,
}

impl Options for Opts {
    type Conf = Config;

    fn shared(&self) -> &crate::opts::Opts { &self.shared }

    fn config(&self) -> Self::Conf {
        Config {
            run_chat: self.chat,
            run_downpour: self.downpour,
            threaded: self.threaded_daemons,
        }
    }
}

impl From<crate::Config<Config>> for crate::Config<()> {
    fn from(config: crate::Config<Config>) -> Self {
        crate::Config {
            data_dir: config.data_dir,
            msg_endpoint: config.msg_endpoint,
            ctl_endpoint: config.ctl_endpoint,
            rpc_endpoint: config.rpc_endpoint,
            ext_endpoint: config.ext_endpoint,
            store_endpoint: config.store_endpoint,
            chat_endpoint: config.chat_endpoint,
            ext: (),
        }
    }
}
