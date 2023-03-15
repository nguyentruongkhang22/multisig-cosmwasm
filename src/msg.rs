use cosmwasm_schema::{ cw_serde };
use cosmwasm_std::{ CosmosMsg, Addr };
use cw3::{ Vote, DepositInfo };
use cw_utils::{ Threshold, Duration, Expiration };

use crate::state::Voter;

#[cw_serde]
pub struct InstantiateMsg {
  pub voters: Vec<Voter>,
  pub threshold: Threshold,
  pub max_voting_period: Duration,
}

#[cw_serde]
pub enum ExecuteMsg {
  // CreateProposal creates a new proposal
  CreateProposal {
    title: String,
    description: String,
    msgs: Vec<CosmosMsg>,
    deposit_info: Option<DepositInfo>,
    expires: Option<Expiration>,
  },
  // Vote casts a vote on a proposal
  Vote {
    proposal_id: u64,
    vote: Vote,
  },
  // ExecuteProposal executes a proposal
  ExecuteProposal {
    proposal_id: u64,
  },
  // CloseProposal closes a proposal
  CloseProposal {
    proposal_id: u64,
  },

  RemoveVoter {
    voter: Addr,
  },
}

// #[derive(QueryResponses)]
#[cw_serde]
pub enum QueryMsg {
  // GetCount returns the current count as a json-encoded number
  GetProposal {
    proposal_id: u64,
  },

  GetCurrentId {},
}