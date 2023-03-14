#[cfg(not(feature = "library"))]
pub mod contract {
  // version info for migration info
  use super::*;
  const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
  const CONTRACT_NAME: &str = "crates.io:cw-template";
  #[cfg(not(feature = "library"))]
  use cosmwasm_std::entry_point;
  use cosmwasm_std::{ to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult };
  use cw2::set_contract_version;

  use crate::error::ContractError;
  use crate::msg::{ ExecuteMsg, InstantiateMsg };
  use crate::state::{ Config, CONFIG };

  #[entry_point]
  pub fn instantiate(deps: DepsMut, _env: Env, info: MessageInfo, msg: InstantiateMsg) -> Result<Response, ContractError> {
    let total_weight = msg.voters
      .iter()
      .map(|voter| voter.weight)
      .sum();

    let config = Config {
      max_voting_period: msg.max_voting_period,
      threshold: msg.threshold,
      total_weight,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::default())
  }

  #[entry_point]
  pub fn execute(deps: DepsMut, _env: Env, info: MessageInfo, msg: ExecuteMsg) -> Result<Response, ContractError> {
    match msg {
      ExecuteMsg::CreateProposal { title, description, msgs } => { unimplemented!() }
      ExecuteMsg::Vote { proposal_id, vote } => { unimplemented!() }
      ExecuteMsg::ExecuteProposal { proposal_id } => { unimplemented!() }
      ExecuteMsg::CloseProposal { proposal_id } => { unimplemented!() }
    }
  }
  //   #[entry_point]
  //   pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
  //     match msg {
  //       QueryMsg::GetCount {} => to_binary(&query::count(deps)?),
  //     }
  //   }
}

pub mod execute {
  use cosmwasm_std::{ DepsMut, MessageInfo, Response, Env, ensure };
  use cw3::{ Proposal, DepositInfo, Ballot };
  use cw_utils::Expiration;

  use crate::{ error::ContractError, state::{ PROPOSALS, next_id, CONFIG, VOTERS, BALLOTS }, helpers::qualified_to_vote };

  pub fn create_proposal(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    title: String,
    description: String,
    deposit_info: Option<DepositInfo>,
    expires: Option<Expiration>
  ) -> Result<Response, ContractError> {
    ensure!(qualified_to_vote(deps.as_ref(), &info.clone().sender), ContractError::Unauthorized {});
    let voter_weight = VOTERS.load(deps.branch().storage, info.clone().sender)?;
    let cfg = CONFIG.load(deps.storage)?;
    let start_height = env.block.height;
    let max_period = cfg.max_voting_period.after(&env.block);

    let prop = Proposal {
      title,
      description,
      start_height,
      msgs: vec![],
      total_weight: cfg.total_weight,
      threshold: cfg.threshold,
      votes: cw3::Votes { yes: 1, no: 0, abstain: 0, veto: 0 },
      status: cw3::Status::Open,
      proposer: info.clone().sender,
      deposit: deposit_info,
      expires: expires.unwrap_or(max_period),
    };
    let id = next_id(deps.storage)?;
    PROPOSALS.save(deps.storage, id, &prop)?;

    let ballot: Ballot = Ballot { weight: voter_weight, vote: cw3::Vote::Yes };
    BALLOTS.save(deps.storage, (id, &info.clone().sender), &ballot)?;

    Ok(Response::default())
  }

  pub fn execute_vote(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    proposal_id: u64,
    vote: cw3::Vote
  ) -> Result<Response, ContractError> {
    ensure!(qualified_to_vote(deps.as_ref(), &info.clone().sender), ContractError::Unauthorized {});
    let voter_weight = VOTERS.load(deps.branch().storage, info.clone().sender)?;

    let mut prop = PROPOSALS.load(deps.storage, proposal_id)?;

    if prop.expires.is_expired(&env.clone().block) {
      return Err(ContractError::Expired {});
    }
    prop.votes.add_vote(vote, voter_weight);
    prop.update_status(&env.block);
    PROPOSALS.save(deps.branch().storage, proposal_id, &prop)?;
    // prop.votes.add_vote(vote, weight)
    Ok(Response::default())
  }

  pub fn close_proposal(mut deps: DepsMut, env: Env, info: MessageInfo, proposal_id: u64) -> Result<Response, ContractError> {
    let mut prop = PROPOSALS.load(deps.as_ref().storage, proposal_id)?;
    ensure!(prop.proposer == info.sender, ContractError::Unauthorized {});
    ensure!(prop.status == cw3::Status::Passed, ContractError::PassedProposal {});
    ensure!(prop.expires.is_expired(&env.block), ContractError::UnexpiredProposal {});

    prop.status = cw3::Status::Rejected;
    PROPOSALS.save(deps.storage, proposal_id, &prop)?;
    Ok(Response::default())
  }

  pub fn execute_proposal(deps: DepsMut, env: Env, proposal_id: u64) -> Result<Response, ContractError> {
    let mut prop = PROPOSALS.load(deps.storage, proposal_id)?;
    prop.update_status(&env.block);
    ensure!(prop.status == cw3::Status::Passed, ContractError::PassedProposal {});
    ensure!(!prop.expires.is_expired(&env.block), ContractError::ExpiredProposal {});

    prop.status = cw3::Status::Executed;
    PROPOSALS.save(deps.storage, proposal_id, &prop)?;

    Ok(Response::default().add_messages(prop.msgs))
  }
}

// pub mod query {
//   use cosmwasm_std::{ Deps, StdResult };

//   use crate::msg::{ GetCountResponse };
//   use crate::state::{ STATE };
//   pub fn count(deps: Deps) -> StdResult<GetCountResponse> {
//     let state = STATE.load(deps.storage)?;
//     Ok(GetCountResponse { count: state.count })
//   }
// }