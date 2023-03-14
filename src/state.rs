use cosmwasm_schema::cw_serde;
use cosmwasm_std::{ Addr, Storage, StdResult };
use cw3::{Proposal, Ballot};
use cw_storage_plus::{ Item, Map };
use cw_utils::{Threshold, Duration};

#[cw_serde]
pub struct Voter {
  pub addr: Addr,
  pub weight: u64,
}
#[cw_serde]
pub struct Config {
    pub threshold: Threshold,
    pub total_weight: u64,
    pub max_voting_period: Duration,
}

pub const PROPOSAL_COUNT: Item<u64> = Item::new("proposal_count");
pub const PROPOSALS: Map<u64, Proposal> = Map::new("proposals");
pub const BALLOTS: Map<(u64, &Addr), Ballot> = Map::new("ballots");
pub const VOTERS: Map<Addr, u64> = Map::new("voters");
pub const CONFIG: Item<Config> = Item::new("config");

pub fn next_id(store: &mut dyn Storage) -> StdResult<u64> {
  let id: u64 = PROPOSAL_COUNT.may_load(store)?.unwrap_or_default() + 1;
  PROPOSAL_COUNT.save(store, &id)?;
  Ok(id)
}
