#[cfg(test)]
mod tests {
  use crate::integration_tests::tests::helpers::{ proper_instantiate, propose, vote, query_proposal, execute_proposal };
  use crate::msg::{ ExecuteMsg };
  use crate::contract::contract;
  use cosmwasm_std::{ Addr, Empty, CosmosMsg, to_binary };
  use cw3::Proposal;
  use cw_multi_test::{ Contract, ContractWrapper };

  pub fn contract_template() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(contract::execute, contract::instantiate, contract::query);
    Box::new(contract)
  }

  mod helpers {
    use cosmwasm_std::{ Addr, Decimal, Coin, Uint128, CosmosMsg, StdError };
    use cw3::{ Proposal, Vote };
    use cw_multi_test::{ App, Executor, AppBuilder, AppResponse };
    use crate::{ msg::{ QueryMsg, InstantiateMsg }, state::Voter, helpers::MultisigContract };
    use super::*;

    const USER: &str = "USER";
    const ADMIN: &str = "ADMIN";
    const NATIVE_DENOM: &str = "denom";

    fn mock_app() -> App {
      AppBuilder::new().build(|router, _, storage| {
        router.bank
          .init_balance(
            storage,
            &Addr::unchecked(USER),
            vec![Coin {
              denom: NATIVE_DENOM.to_string(),
              amount: Uint128::new(1),
            }]
          )
          .unwrap();
      })
    }

    pub fn proper_instantiate() -> (App, MultisigContract) {
      let mut app = mock_app();
      let multisig_id = app.store_code(contract_template());

      let msg = InstantiateMsg {
        voters: vec![
          Voter { addr: Addr::unchecked("voter1".to_string()), weight: 1 },
          Voter { addr: Addr::unchecked("voter2".to_string()), weight: 2 },
          Voter { addr: Addr::unchecked("voter3".to_string()), weight: 3 },
          Voter { addr: Addr::unchecked("voter4".to_string()), weight: 4 },
          Voter { addr: Addr::unchecked("voter5".to_string()), weight: 5 },
          Voter { addr: Addr::unchecked("voter6".to_string()), weight: 6 }
        ],
        threshold: cw_utils::Threshold::AbsolutePercentage { percentage: Decimal::percent(51) },
        max_voting_period: cw_utils::Duration::Time(60 * 60 * 24), // 1 day
      };
      let multisig_contract_addr = app
        .instantiate_contract(multisig_id, Addr::unchecked(ADMIN), &msg, &[], "test", None)
        .unwrap();

      let multisig_contract = MultisigContract(multisig_contract_addr);

      (app, multisig_contract)
    }

    pub fn query_proposal(app: &App, contract: &Addr, proposal_id: u64) -> Proposal {
      app.wrap().query_wasm_smart(contract, &(QueryMsg::GetProposal { proposal_id })).unwrap()
    }

    pub fn query_current_id(app: &App, contract: &Addr) -> u64 {
      app.wrap().query_wasm_smart(contract, &(QueryMsg::GetCurrentId {})).unwrap()
    }

    pub fn propose(
      app: &mut App,
      contract: &Addr,
      proposer: &Addr,
      title: &str,
      description: &str,
      msgs: Vec<CosmosMsg>
    ) -> u64 {
      let msg = ExecuteMsg::CreateProposal {
        title: title.to_string(),
        description: description.to_string(),
        msgs,
        deposit_info: None,
        expires: None,
      };

      app.execute_contract(proposer.clone(), contract.clone(), &msg, &[]).unwrap();
      let proposal_id = query_current_id(app, contract);
      proposal_id
    }

    pub fn vote(
      app: &mut App,
      contract: &Addr,
      sender: &Addr,
      proposal_id: u64,
      vote: Vote
    ) -> Result<AppResponse, StdError> {
      let msg = ExecuteMsg::Vote { proposal_id, vote };
      let res = app.execute_contract(sender.clone(), contract.clone(), &msg, &[]).unwrap();
      Ok(res)
    }

    pub fn execute_proposal(app: &mut App, contract: &Addr, proposal_id: u64) -> Result<AppResponse, StdError> {
      let msg = ExecuteMsg::ExecuteProposal { proposal_id };
      let res = app.execute_contract(Addr::unchecked(ADMIN.to_string()), contract.clone(), &msg, &[]).unwrap();
      Ok(res)
    }
  }

  #[test]
  fn proposal() {
    let (mut app, multisig_contract) = proper_instantiate();
    let rm_msg = ExecuteMsg::RemoveVoter { voter: Addr::unchecked("voter2".to_string()) };
    let msg = CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
      contract_addr: multisig_contract.addr().to_string(),
      msg: to_binary(&rm_msg).unwrap(),
      funds: vec![],
    });

    // create proposal
    let proposal_id = propose(
      &mut app,
      &multisig_contract.addr(),
      &Addr::unchecked("voter1".to_string()),
      "test",
      "test",
      vec![msg]
    );

    // execute vote
    vote(
      &mut app,
      &multisig_contract.addr(),
      &Addr::unchecked("voter2".to_string()),
      proposal_id,
      cw3::Vote::Yes
    ).unwrap();

    vote(
      &mut app,
      &multisig_contract.addr(),
      &Addr::unchecked("voter3".to_string()),
      proposal_id,
      cw3::Vote::No
    ).unwrap();

    vote(
      &mut app,
      &multisig_contract.addr(),
      &Addr::unchecked("voter6".to_string()),
      proposal_id,
      cw3::Vote::Yes
    ).unwrap();

    vote(
      &mut app,
      &multisig_contract.addr(),
      &Addr::unchecked("voter5".to_string()),
      proposal_id,
      cw3::Vote::Yes
    ).unwrap();

    let proposal: Proposal = query_proposal(&app, &multisig_contract.addr(), proposal_id);
    println!(" -- proposal: {:?}", proposal);

    execute_proposal(&mut app, &multisig_contract.addr(), proposal_id).unwrap();
  }
}