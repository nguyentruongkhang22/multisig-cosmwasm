use cosmwasm_schema::{ cw_serde, QueryResponses };
use cosmwasm_std::CosmosMsg;
use cw3::Vote;
use cw_utils::{ Threshold, Duration };

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
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
  // GetCount returns the current count as a json-encoded number
  #[returns(GetCountResponse)] GetCount {},
}

// We define a custom struct for each query response
#[cw_serde]
pub struct GetCountResponse {
  pub count: i32,
}