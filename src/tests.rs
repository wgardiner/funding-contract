#[cfg(test)]
mod tests {
    use crate::contract::{get_unique_votes, calculate_distributions, handle, init, query};
    use crate::error::ContractError;
    use crate::msg::{
        CheckDistributionsResponse, CreateProposalResponse, HandleMsg, InitMsg,
        ProposalListResponse, ProposalStateResponse, QueryMsg, StateResponse,
    };
    use crate::state::{config_read, Vote, Proposal};
    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{coins, from_binary, Api, Extern, HumanAddr};

    fn default_init_msg() -> InitMsg {
        let env = mock_env();
        InitMsg {
            name: "My Funding Round".to_string(),
            proposer_whitelist: vec![
                HumanAddr::from("proposer_0"),
                HumanAddr::from("proposer_1"),
                HumanAddr::from("proposer_2"),
            ],
            voter_whitelist: vec![
                HumanAddr::from("voter_0"),
                HumanAddr::from("voter_1"),
                HumanAddr::from("voter_2"),
            ],
            proposal_period_start: Some(env.block.time),
            proposal_period_end: Some(env.block.time + 86400),
            voting_period_start: Some(env.block.time + 86400 * 2),
            voting_period_end: Some(env.block.time + 86400 * 5),
        }
    }

    fn mock_init(mut deps: &mut Extern<MockStorage, MockApi, MockQuerier>, msg: InitMsg) {
        let info = mock_info("owner", &coins(1000, "earth"));
        let _res = init(&mut deps, mock_env(), info, msg).unwrap();
    }

    fn default_proposal_msg() -> HandleMsg {
        HandleMsg::CreateProposal {
            name: "My proposal".to_string(),
            recipient: HumanAddr::from("proposal_recipient"),
            description: "The proposal description".to_string(),
            tags: "one two three".to_string(),
        }
    }

    fn mock_proposal(mut deps: &mut Extern<MockStorage, MockApi, MockQuerier>, msg: HandleMsg) {
        let info = mock_info("proposer_0", &coins(1000, "earth"));
        let _res = handle(&mut deps, mock_env(), info, msg).unwrap();
    }

    #[test]
    fn fails_initialization_invalid_proposal_period() {
        // TODO: Test that proposal start is before the end.
    }

    #[test]
    fn fails_initialization_invalid_voting_period() {
        // TODO: Test that vote start is before the end.
    }

    #[test]
    fn fails_initialization_voting_before_proposal() {
        // TODO: Test that proposal period ends before the voting period.
    }

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = default_init_msg();
        let info = mock_info("owner", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(&deps, mock_env(), QueryMsg::GetState {}).unwrap();
        let value: StateResponse = from_binary(&res).unwrap();
        // let value: CountResponse = from_binary(&res).unwrap();
        // println!("full response {:?}", value);
        // println!("{:?}", value.proposer_whitelist[0]);
        assert_eq!(3, value.proposer_whitelist.len());
        assert_eq!(HumanAddr::from("proposer_0"), value.proposer_whitelist[0]);
        assert_eq!(3, value.voter_whitelist.len());
        assert_eq!(HumanAddr::from("voter_0"), value.voter_whitelist[0]);
        // assert_eq!(17, value.count);
        assert_eq!("My Funding Round", value.name);
    }

    #[test]
    fn fails_create_proposal_invalid_address() {
        let mut deps = mock_dependencies(&[]);
        mock_init(&mut deps, default_init_msg());

        // create proposal.
        let proposal_msg = default_proposal_msg();

        // try to create a proposal as "any_user"
        let info = mock_info("any_user", &coins(1000, "earth"));
        let res = handle(&mut deps, mock_env(), info, proposal_msg);
        match res {
            Err(ContractError::Unauthorized { list_type: _ }) => {}
            _ => panic!("Must return error"),
        }

        // proposal should not have been created.
        let state = config_read(&deps.storage).load().unwrap();
        assert_eq!(0, state.proposals.len(),);
    }

    #[test]
    fn fails_create_proposal_invalid_period() {
        let mut deps = mock_dependencies(&[]);
        let env = mock_env();

        // change proposal time so it has already expired.
        let mut msg = default_init_msg();
        msg.proposal_period_start = Some(env.block.time - 86400 * 5);
        msg.proposal_period_end = Some(env.block.time - 86400 * 1);
        mock_init(&mut deps, msg);

        // create proposal.
        let proposal_msg = default_proposal_msg();

        let info = mock_info("proposer_0", &coins(1000, "earth"));
        let res = handle(&mut deps, mock_env(), info, proposal_msg);

        match res {
            Err(ContractError::InvalidPeriod { period_type: _ }) => {}
            _ => panic!("Must return error"),
        }

        // proposal should not have been created.
        let state = config_read(&deps.storage).load().unwrap();
        assert_eq!(0, state.proposals.len(),);
    }

    #[test]
    fn fails_create_proposal_insufficient_data() {
        // TODO: Test creating a proposal with missing data.
    }

    #[test]
    fn create_proposal_no_proposer_list() {
        let mut deps = mock_dependencies(&[]);

        // modify init message to empty proposer whitelist.
        let mut msg = default_init_msg();
        msg.proposer_whitelist = Vec::new();
        mock_init(&mut deps, msg);

        // create proposal.
        let proposal_msg = default_proposal_msg();

        // try to create a proposal as "any_user"
        let info = mock_info("any_user", &coins(1000, "earth"));
        let res = handle(&mut deps, mock_env(), info, proposal_msg).unwrap();
        let data = res.data.unwrap();
        let value: CreateProposalResponse = from_binary(&data).unwrap();
        assert_eq!(0, value.proposal_id);

        // proposal should be created.
        let state = config_read(&deps.storage).load().unwrap();
        assert_eq!(1, state.proposals.len(),);

        // test Proposal List query response.
        let res = query(&mut deps, mock_env(), QueryMsg::ProposalList {}).unwrap();
        let value: ProposalListResponse = from_binary(&res).unwrap();
        assert_eq!(1, value.proposals.len());
        assert_eq!("My proposal", value.proposals[0].name);
        let recipient = deps
            .api
            .canonical_address(&HumanAddr::from("proposal_recipient"))
            .unwrap();
        assert_eq!(recipient, value.proposals[0].recipient);

        // test Proposal State query response.
        let res = query(
            &mut deps,
            mock_env(),
            QueryMsg::ProposalState { proposal_id: 0 },
        )
        .unwrap();
        let value: ProposalStateResponse = from_binary(&res).unwrap();
        assert_eq!("My proposal", value.proposal.name);
        assert_eq!(recipient, value.proposal.recipient);
        assert_eq!(0, value.votes.len());
    }

    #[test]
    fn create_proposal() {
        let mut deps = mock_dependencies(&[]);
        mock_init(&mut deps, default_init_msg());

        // Create proposal.
        let proposal_msg = HandleMsg::CreateProposal {
            name: "My proposal".to_string(),
            recipient: HumanAddr::from("proposal_recipient"),
            description: "The proposal description".to_string(),
            tags: "one two three".to_string(),
        };

        let info = mock_info("proposer_0", &coins(1000, "earth"));
        let res = handle(&mut deps, mock_env(), info, proposal_msg).unwrap();
        let data = res.data.unwrap();
        let value: CreateProposalResponse = from_binary(&data).unwrap();
        assert_eq!(0, value.proposal_id);

        // proposal should be created.
        let state = config_read(&deps.storage).load().unwrap();
        assert_eq!(1, state.proposals.len(),);

        // test Proposal List query response.
        let res = query(&mut deps, mock_env(), QueryMsg::ProposalList {}).unwrap();
        let value: ProposalListResponse = from_binary(&res).unwrap();
        assert_eq!(1, value.proposals.len());
        assert_eq!("My proposal", value.proposals[0].name);
        let recipient = deps
            .api
            .canonical_address(&HumanAddr::from("proposal_recipient"))
            .unwrap();
        assert_eq!(recipient, value.proposals[0].recipient);

        // test Proposal State query response.
        let res = query(
            &mut deps,
            mock_env(),
            QueryMsg::ProposalState { proposal_id: 0 },
        )
        .unwrap();
        let value: ProposalStateResponse = from_binary(&res).unwrap();
        assert_eq!("My proposal", value.proposal.name);
        assert_eq!(recipient, value.proposal.recipient);
        assert_eq!(0, value.votes.len());
    }

    #[test]
    fn fails_create_vote_invalid_address() {
        let mut deps = mock_dependencies(&[]);
        mock_init(&mut deps, default_init_msg());
        mock_proposal(&mut deps, default_proposal_msg());

        // create vote.
        let vote_msg = HandleMsg::CreateVote { proposal_id: 1 };

        // try to create a vote as "any user"
        let info = mock_info("any_user", &coins(1000, "earth"));

        // set the time to the voting period.
        let mut env = mock_env();
        env.block.time = env.block.time + 86400 * 3;

        // send message.
        let res = handle(&mut deps, env, info, vote_msg);
        match res {
            Err(ContractError::Unauthorized { list_type: _ }) => {}
            _ => panic!("Must return error"),
        }

        // vote should not be created.
        let state = config_read(&deps.storage).load().unwrap();
        assert_eq!(0, state.votes.len(),);
    }

    #[test]
    fn fails_create_vote_invalid_proposal() {
        let mut deps = mock_dependencies(&[]);
        mock_init(&mut deps, default_init_msg());
        mock_proposal(&mut deps, default_proposal_msg());
        mock_proposal(&mut deps, default_proposal_msg());

        // create vote.
        // use an invalid proposal id.
        let vote_msg = HandleMsg::CreateVote { proposal_id: 2 };
        let info = mock_info("voter_0", &coins(1000, "earth"));

        // set the time to the voting period.
        let mut env = mock_env();
        env.block.time = env.block.time + 86400 * 3;

        // send message.
        let res = handle(&mut deps, env, info, vote_msg);
        match res {
            Err(ContractError::InvalidProposal { id: _ }) => {}
            _ => panic!("Must return error"),
        }

        // vote should not be created.
        let state = config_read(&deps.storage).load().unwrap();
        assert_eq!(0, state.votes.len(),);
    }

    #[test]
    fn fails_create_vote_invalid_period() {
        let mut deps = mock_dependencies(&[]);
        mock_init(&mut deps, default_init_msg());
        mock_proposal(&mut deps, default_proposal_msg());

        // create vote.
        let vote_msg = HandleMsg::CreateVote { proposal_id: 1 };
        let info = mock_info("voter_0", &coins(1000, "earth"));

        // set the time to the proposal period.
        let mut env = mock_env();
        env.block.time = env.block.time + 86400 * 1;

        // send message.
        let res = handle(&mut deps, env, info, vote_msg);
        match res {
            Err(ContractError::InvalidPeriod { period_type: _ }) => {}
            _ => panic!("Must return error"),
        }

        // vote should not be created.
        let state = config_read(&deps.storage).load().unwrap();
        assert_eq!(0, state.votes.len(),);
    }

    #[test]
    fn create_vote_no_voter_list() {
        let mut deps = mock_dependencies(&[]);

        // modify init message to empty voter whitelist.
        let mut msg = default_init_msg();
        msg.voter_whitelist = Vec::new();
        mock_init(&mut deps, msg);

        mock_proposal(&mut deps, default_proposal_msg());

        // create vote.
        let vote_msg = HandleMsg::CreateVote { proposal_id: 0 };

        // try to create a vote as "any user"
        let info = mock_info("any_user", &coins(1000, "earth"));

        // set the time to the voting period.
        let mut env = mock_env();
        env.block.time = env.block.time + 86400 * 3;

        // send message.
        let _res = handle(&mut deps, env, info, vote_msg).unwrap();

        // vote should be created.
        let state = config_read(&deps.storage).load().unwrap();
        assert_eq!(1, state.votes.len(),);

        // check voter address.
        let voter = deps
            .api
            .canonical_address(&HumanAddr("any_user".to_string()))
            .unwrap();
        assert_eq!(voter, state.votes[0].voter);

        // check amount.
        assert_eq!(coins(1000, "earth"), state.votes[0].amount);

        // test Proposal State query response.
        let res = query(
            &mut deps,
            mock_env(),
            QueryMsg::ProposalState { proposal_id: 0 },
        )
        .unwrap();
        let value: ProposalStateResponse = from_binary(&res).unwrap();
        assert_eq!("My proposal", value.proposal.name);
        assert_eq!(1, value.votes.len());
        assert_eq!(voter, value.votes[0].voter);
        assert_eq!(coins(1000, "earth"), value.votes[0].amount);
    }

    #[test]
    fn create_vote() {
        let mut deps = mock_dependencies(&[]);
        mock_init(&mut deps, default_init_msg());
        mock_proposal(&mut deps, default_proposal_msg());
        mock_proposal(&mut deps, default_proposal_msg());
        mock_proposal(&mut deps, default_proposal_msg());

        // create vote.
        let vote_msg = HandleMsg::CreateVote { proposal_id: 2 };
        let info = mock_info("voter_0", &coins(1000, "earth"));

        // set the time to the voting period.
        let mut env = mock_env();
        env.block.time = env.block.time + 86400 * 3;

        // send message.
        let _res = handle(&mut deps, env, info, vote_msg).unwrap();

        // vote should be created.
        let state = config_read(&deps.storage).load().unwrap();
        assert_eq!(1, state.votes.len(),);

        // check voter address.
        let voter = deps
            .api
            .canonical_address(&HumanAddr("voter_0".to_string()))
            .unwrap();
        assert_eq!(voter, state.votes[0].voter);

        // check amount.
        assert_eq!(coins(1000, "earth"), state.votes[0].amount);

        // test Proposal State query response.
        let res = query(
            &mut deps,
            mock_env(),
            QueryMsg::ProposalState { proposal_id: 2 },
        )
        .unwrap();
        let value: ProposalStateResponse = from_binary(&res).unwrap();
        assert_eq!("My proposal", value.proposal.name);
        assert_eq!(1, value.votes.len());
        assert_eq!(voter, value.votes[0].voter);
        assert_eq!(coins(1000, "earth"), value.votes[0].amount);
    }

    #[test]
    fn unique_votes() {
        let deps = mock_dependencies(&[]);
        let votes: Vec<Vote> = vec![
            Vote {
                voter: deps
                    .api
                    .canonical_address(&HumanAddr("voter_0".to_string()))
                    .unwrap(),
                proposal: 0,
                amount: coins(1, "earth"),
            },
            Vote {
                voter: deps
                    .api
                    .canonical_address(&HumanAddr("voter_1".to_string()))
                    .unwrap(),
                proposal: 0,
                amount: coins(1, "earth"),
            },
            Vote {
                voter: deps
                    .api
                    .canonical_address(&HumanAddr("voter_0".to_string()))
                    .unwrap(),
                proposal: 1,
                amount: coins(1, "earth"),
            },
            Vote {
                voter: deps
                    .api
                    .canonical_address(&HumanAddr("voter_0".to_string()))
                    .unwrap(),
                proposal: 0,
                amount: coins(1, "earth"),
            },
        ];
        let result = get_unique_votes(&votes);
        assert_eq!(result.len(), 3);
        let pretty_result: Vec<_> = result
            .iter()
            .map(|x| {
                (
                    x.proposal,
                    deps.api.human_address(&x.voter).unwrap(),
                    x.amount[0].amount.u128(),
                )
            })
            .collect();
        let votes_by_voter_0: Vec<_> = pretty_result
            .iter()
            .filter(|x| &*x.1 == "voter_0")
            .collect();
        assert_eq!(votes_by_voter_0.len(), 2);
        let votes_by_voter_1: Vec<_> = pretty_result
            .iter()
            .filter(|x| &*x.1 == "voter_1")
            .collect();
        assert_eq!(votes_by_voter_1.len(), 1);
        assert_eq!(votes_by_voter_1[0].2, 1);

        let votes_for_prop_0_by_voter_0: Vec<_> = pretty_result
            .iter()
            .filter(|x| &*x.1 == "voter_0" && x.0 == 0)
            .collect();
        assert_eq!(votes_for_prop_0_by_voter_0.len(), 1);
        assert_eq!(votes_for_prop_0_by_voter_0[0].2, 2);
        // println!("{:#?}", pretty_result);
    }

    #[test]
    fn test_calculate_distribution() {}

    #[test]
    fn fail_check_distributions_proposal_period() {
        let mut deps = mock_dependencies(&[]);
        mock_init(&mut deps, default_init_msg());

        // anyone can check the distributions.
        let info = mock_info("any_user", &coins(1000, "earth"));

        // set the time to the proposal period.
        // cannot check distributions during proposal period.
        let mut env = mock_env();
        env.block.time = env.block.time + 86400 * 1;

        // send message.
        let msg = HandleMsg::CheckDistributions {};
        let res = handle(&mut deps, env, info, msg);
        match res {
            Err(ContractError::InvalidPeriod { period_type: _ }) => {}
            _ => panic!("Must return error"),
        }
    }

    #[test]
    fn check_distributions() {
        let mut deps = mock_dependencies(&[]);
        mock_init(&mut deps, default_init_msg());
        mock_proposal(&mut deps, default_proposal_msg());
        mock_proposal(&mut deps, default_proposal_msg());
        mock_proposal(&mut deps, default_proposal_msg());

        // anyone can check the distributions.
        let info = mock_info("any_user", &coins(1000, "earth"));

        // set the time to the voting period.
        let mut env = mock_env();
        env.block.time = env.block.time + 86400 * 3;

        // send message.
        let msg = HandleMsg::CheckDistributions {};
        let res = handle(&mut deps, env, info, msg).unwrap();
        let data = res.data.unwrap();
        let value: CheckDistributionsResponse = from_binary(&data).unwrap();

        // assert there is a ProposalDistribution for every proposal.
        let state = config_read(&deps.storage).load().unwrap();
        assert_eq!(state.proposals.len(), value.distributions.len());
    }

    #[test]
    fn fail_distribute_funds_unauthorized() {
        let mut deps = mock_dependencies(&[]);
        mock_init(&mut deps, default_init_msg());

        // try to distribute funds as any user.
        let info = mock_info("any_user", &coins(1000, "earth"));

        // set the time to the voting period.
        // cannot check distributions during voting period.
        let mut env = mock_env();
        env.block.time = env.block.time + 86400 * 6;

        // send message.
        let msg = HandleMsg::DistributeFunds {};
        let res = handle(&mut deps, env, info, msg);
        match res {
            Err(ContractError::Unauthorized { list_type: _ }) => {}
            _ => panic!("Must return error"),
        }
    }

    #[test]
    fn fail_distribute_funds_invalid_period() {
        let mut deps = mock_dependencies(&[]);
        mock_init(&mut deps, default_init_msg());

        // owner can distribute funds.
        let info = mock_info("owner", &coins(1000, "earth"));

        // set the time to the voting period.
        // cannot check distributions during voting period.
        let mut env = mock_env();
        env.block.time = env.block.time + 86400 * 3;

        // send message.
        let msg = HandleMsg::DistributeFunds {};
        let res = handle(&mut deps, env, info, msg);
        match res {
            Err(ContractError::InvalidPeriod { period_type: _ }) => {}
            _ => panic!("Must return error"),
        }
    }

    #[test]
    fn distribute_funds() {
        let mut deps = mock_dependencies(&[]);
        mock_init(&mut deps, default_init_msg());
        mock_proposal(&mut deps, default_proposal_msg());
        mock_proposal(&mut deps, default_proposal_msg());
        mock_proposal(&mut deps, default_proposal_msg());

        // owner can distribute funds.
        let info = mock_info("owner", &coins(1000, "earth"));

        // set the time to after the voting period.
        let mut env = mock_env();
        env.block.time = env.block.time + 86400 * 6;

        // send message.
        let msg = HandleMsg::DistributeFunds {};
        let res = handle(&mut deps, env, info, msg).unwrap();
        let data = res.data.unwrap();
        let value: CheckDistributionsResponse = from_binary(&data).unwrap();

        // assert there is a ProposalDistribution for every proposal.
        let state = config_read(&deps.storage).load().unwrap();
        assert_eq!(state.proposals.len(), value.distributions.len());

        // TODO: Assert that proposal recipients got funds.
    }

    #[test]
    fn test_calculate_distributions() {
        let deps = mock_dependencies(&[]);
        let votes: Vec<Vote> = vec![
            Vote {
                voter: deps
                    .api
                    .canonical_address(&HumanAddr("voter_0".to_string()))
                    .unwrap(),
                proposal: 0,
                amount: coins(1, "earth"),
            },
            Vote {
                voter: deps
                    .api
                    .canonical_address(&HumanAddr("voter_1".to_string()))
                    .unwrap(),
                proposal: 0,
                amount: coins(4, "earth"),
            },
            Vote {
                voter: deps
                    .api
                    .canonical_address(&HumanAddr("voter_2".to_string()))
                    .unwrap(),
                proposal: 1,
                amount: coins(9, "earth"),
            },
            Vote {
                voter: deps
                    .api
                    .canonical_address(&HumanAddr("voter_0".to_string()))
                    .unwrap(),
                proposal: 1,
                amount: coins(16, "earth"),
            },
        ];
        let proposals = vec![
            Proposal {
                id: 0,
                name: "Proposal 0".to_string(),
                recipient: deps.api.canonical_address(&HumanAddr("recipient_0".to_string())).unwrap(),
                description: "an okay proposal".to_string(),
                tags: "money".to_string(),
            },
            Proposal {
                id: 1,
                name: "Proposal 1".to_string(),
                recipient: deps.api.canonical_address(&HumanAddr("recipient_1".to_string())).unwrap(),
                description: "an better proposal".to_string(),
                tags: "stuffed animals, parrots".to_string(),
            },
        ];
        let result = calculate_distributions(votes, proposals, coins(50, "shell"));
        println!("{:#?}", result);
    }
}
