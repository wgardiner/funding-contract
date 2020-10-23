use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized: Sender address not in {list_type:?} list")]
    Unauthorized { list_type: String },

    #[error("Invalid {period_type:?} period")]
    InvalidPeriod { period_type: String },

    #[error("Invalid proposal id: {id:?}")]
    InvalidProposal { id: u32 },
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
