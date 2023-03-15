use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
  #[error("{0}")] Std(#[from] StdError),

  #[error("Unauthorized")] Unauthorized {},

  #[error("Expired")] Expired {},

  #[error("Passed proposal")] PassedProposal {},

  #[error("Proposal is not Passed")] OpenProposal {},

  #[error("Proposal is not expired")] UnexpiredProposal {},

  #[error("Proposal is expired")] ExpiredProposal {},

  #[error("Voter has already voted")] VotedVoter {},

  // Add any other custom errors you like here.
  // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}