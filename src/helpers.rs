use cosmwasm_std::{ Deps, Addr, StdResult, CosmosMsg, to_binary, WasmMsg, ensure };
use schemars::JsonSchema;
use serde::{ Serialize, Deserialize };

use crate::{ state::{ VOTERS, BALLOTS }, msg::ExecuteMsg, ContractError };

pub fn qualified_to_vote(deps: Deps, voter: &Addr, proposal_id: u64) -> Result<bool, ContractError> {
  let voted = BALLOTS.may_load(deps.storage, (proposal_id, &voter.clone()));
  let is_voted = match voted {
    Ok(Some(_)) => true,
    Ok(None) => false,
    Err(_) => {
      return Err(ContractError::Unauthorized {});
    }
  };
  ensure!(!is_voted, ContractError::VotedVoter {});

  let weight = VOTERS.may_load(deps.storage, voter.clone()).unwrap();

  match weight {
    Some(weight) => {
      if weight >= 1 {
        return Ok(true);
      } else {
        return Err(ContractError::Unauthorized {});
      }
    }
    None => {
      return Err(ContractError::Unauthorized {});
    }
  }
}

pub fn qualified_to_propose(deps: Deps, voter: &Addr) -> bool {
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MultisigContract(pub Addr);

impl MultisigContract {
  pub fn addr(&self) -> Addr {
    self.0.clone()
  }

  pub fn call<T: Into<ExecuteMsg>>(&self, msg: T) -> StdResult<CosmosMsg> {
    let msg = to_binary(&msg.into())?;
    Ok(
      (WasmMsg::Execute {
        contract_addr: self.addr().into(),
        msg,
        funds: vec![],
      }).into()
    )
  }
}