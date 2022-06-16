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

use crate::FailureCode;

/// We need this wrapper type to be compatible with Storm Node having multiple message buses
#[derive(Clone, Debug, Display, From, Api)]
#[api(encoding = "strict")]
#[non_exhaustive]
pub(crate) enum BusMsg {
    #[api(type = 4)]
    #[display(inner)]
    #[from]
    Rpc(RpcMsg),
}

impl rpc::Request for BusMsg {}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Display, From)]
#[derive(NetworkEncode, NetworkDecode)]
#[display(inner)]
pub enum RpcMsg {
    RegisterApp(RegisterAppReq),

    // Responses to CLI
    // ----------------
    #[display("success({0})")]
    Success,

    #[display("failure({0:#})")]
    #[from]
    Failure(rpc::Failure<FailureCode>),
}

impl From<presentation::Error> for RpcMsg {
    fn from(err: presentation::Error) -> Self {
        RpcMsg::Failure(rpc::Failure {
            code: rpc::FailureCode::Presentation,
            info: format!("{}", err),
        })
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Display, Default, From)]
#[derive(NetworkEncode, NetworkDecode)]
#[display("register_app({bifrost_code:#06X})")]
pub struct RegisterAppReq {
    pub bifrost_code: u16,
}
