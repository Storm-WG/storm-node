// Storm node providing distributed storage & messaging for lightning network.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use internet2::presentation;
use microservices::rpc;
use storm_rpc::FailureCode;

/// We need this wrapper type to be compatible with Storm Node having multiple message buses
#[derive(Clone, Debug, Display, From, Api)]
#[api(encoding = "strict")]
#[non_exhaustive]
pub(crate) enum BusMsg {
    #[api(type = 5)]
    #[display(inner)]
    #[from]
    App(AppMsg),
}

impl rpc::Request for BusMsg {}

#[derive(Clone, Debug, Display, From)]
#[derive(NetworkEncode, NetworkDecode)]
#[display(inner)]
#[non_exhaustive]
pub enum AppMsg {
    RegisterApp(RegisterAppReq),

    // Responses to CLI
    // ----------------
    #[display("success({0})")]
    Success,

    #[display("failure({0:#})")]
    #[from]
    Failure(rpc::Failure<FailureCode>),
}

impl From<presentation::Error> for AppMsg {
    fn from(err: presentation::Error) -> Self {
        AppMsg::Failure(rpc::Failure {
            code: rpc::FailureCode::Presentation,
            info: format!("{}", err),
        })
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Display, Default)]
#[derive(NetworkEncode, NetworkDecode)]
#[display("register_app({bifrost_code:#06X})")]
pub struct RegisterAppReq {
    pub bifrost_code: u16,
}
