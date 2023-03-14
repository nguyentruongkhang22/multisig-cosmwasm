use cosmwasm_std::{ Deps, Addr, StdResult };

use crate::state::VOTERS;

pub fn qualified_to_vote(deps: Deps, voter: &Addr) -> bool {
  let weight = VOTERS.may_load(deps.storage, voter.clone()).unwrap();
  match weight {
    Some(weight) => {
      if weight > 0 {
        return true;
      }
    }
    None => {
      return false;
    }
  }
  false
}