#[cfg(not(feature = "library"))]
pub mod contract {
  // version info for migration info
  use super::execute::{ create_proposal, execute_vote, execute_proposal, close_proposal };
  const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
  const CONTRACT_NAME: &str = "crates.io:cw-template";
  #[cfg(not(feature = "library"))]
  use cosmwasm_std::entry_point;
  use cosmwasm_std::{ to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, ensure };
  use cw2::set_contract_version;

  use crate::error::ContractError;
  use crate::msg::{ ExecuteMsg, InstantiateMsg, QueryMsg };
  use crate::state::{ Config, CONFIG, PROPOSALS, VOTERS, PROPOSAL_COUNT };

  #[entry_point]
  pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg
  ) -> Result<Response, ContractError> {
    let total_weight = msg.voters
      .iter()
      .map(|voter| voter.weight)
      .sum();

    let config = Config {
      max_voting_period: msg.max_voting_period,
      threshold: msg.threshold,
      total_weight,
    };

    msg.voters.iter().for_each(|v| {
      VOTERS.save(deps.storage, v.clone().addr, &v.weight).unwrap();
    });
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::default())
  }

  #[entry_point]
  pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> Result<Response, ContractError> {
    match msg {
      ExecuteMsg::CreateProposal { title, description, deposit_info, expires, msgs } =>
        create_proposal(deps, env, info, title, description, deposit_info, expires, msgs),
      ExecuteMsg::Vote { proposal_id, vote } => execute_vote(deps, env, info, proposal_id, vote),
      ExecuteMsg::ExecuteProposal { proposal_id } => execute_proposal(deps, env, proposal_id),
      ExecuteMsg::CloseProposal { proposal_id } => close_proposal(deps, env, info, proposal_id),
      ExecuteMsg::RemoveVoter { voter } => {
        ensure!(info.sender == env.contract.address, ContractError::Unauthorized {});
        VOTERS.remove(deps.storage, voter);
        Ok(Response::default())
      }
    }
  }
  #[entry_point]
  pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
      QueryMsg::GetProposal { proposal_id } => to_binary(&PROPOSALS.load(deps.storage, proposal_id)?),
      QueryMsg::GetCurrentId {} => {
        let id: u64 = PROPOSAL_COUNT.load(deps.storage)?;
        to_binary(&id)
      }
    }
  }
}

pub mod execute {
  use cosmwasm_std::{ DepsMut, MessageInfo, Response, Env, ensure, CosmosMsg, to_binary };
  use cw3::{ Proposal, DepositInfo, Ballot };
  use cw_utils::Expiration;

  use crate::{
    error::ContractError,
    state::{ PROPOSALS, next_id, CONFIG, VOTERS, BALLOTS },
    helpers::{ qualified_to_vote, qualified_to_propose },
  };

  pub fn create_proposal(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    title: String,
    description: String,
    deposit_info: Option<DepositInfo>,
    expires: Option<Expiration>,
    msgs: Vec<CosmosMsg>
  ) -> Result<Response, ContractError> {
    ensure!(qualified_to_propose(deps.as_ref(), &info.clone().sender), ContractError::Unauthorized {});
    let voter_weight = VOTERS.load(deps.branch().storage, info.clone().sender)?;
    let cfg = CONFIG.load(deps.storage)?;
    let start_height = env.block.height;
    let max_period = cfg.max_voting_period.after(&env.block);

    let prop = Proposal {
      title,
      description,
      start_height,
      msgs,
      total_weight: cfg.total_weight,
      threshold: cfg.threshold,
      votes: cw3::Votes::yes(voter_weight),
      status: cw3::Status::Open,
      proposer: info.clone().sender,
      deposit: deposit_info,
      expires: expires.unwrap_or(max_period),
    };
    let id = next_id(deps.storage)?;
    PROPOSALS.save(deps.storage, id, &prop)?;

    let ballot: Ballot = Ballot { weight: voter_weight, vote: cw3::Vote::Yes };
    BALLOTS.save(deps.storage, (id, &info.clone().sender), &ballot)?;
    let mut res: Response = Response::default();
    res.data = Some(to_binary(&id).unwrap());

    Ok(res)
  }

  pub fn execute_vote(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    proposal_id: u64,
    vote: cw3::Vote
  ) -> Result<Response, ContractError> {
    qualified_to_vote(deps.as_ref(), &info.clone().sender, proposal_id)?;
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

  pub fn close_proposal(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    proposal_id: u64
  ) -> Result<Response, ContractError> {
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
    ensure!(prop.status == cw3::Status::Passed, ContractError::OpenProposal {});
    ensure!(!prop.expires.is_expired(&env.block), ContractError::ExpiredProposal {});

    prop.status = cw3::Status::Executed;
    PROPOSALS.save(deps.storage, proposal_id, &prop)?;

    Ok(Response::default().add_messages(prop.msgs))
  }
}