#[cfg(test)]
mod tests {
    use crate::contract::{init, handle, query};
    use crate::msg::{StateResponse, HandleMsg, InitMsg, QueryMsg};
    use crate::state::{config_read};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MockStorage, MockApi, MockQuerier};
    use cosmwasm_std::{ Extern, HumanAddr, coins, from_binary};

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
        let info = mock_info("creator", &coins(1000, "earth"));
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
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);
        let env = mock_env();

        let msg = default_init_msg();
        let info = mock_info("creator", &coins(1000, "earth"));

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
        let _res = handle(&mut deps, mock_env(), info, proposal_msg).unwrap();

        // proposal should not have been created.
        let state = config_read(&deps.storage).load().unwrap();
        assert_eq!(
            0,
            state.proposals.len(),
        );
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

        let info = mock_info("creator", &coins(1000, "earth"));
        let _res = handle(&mut deps, mock_env(), info, proposal_msg).unwrap();

        // proposal should not have been created.
        let state = config_read(&deps.storage).load().unwrap();
        assert_eq!(
            0,
            state.proposals.len(),
        );
    }

    #[test]
    fn fails_create_proposal_insufficient_data() {

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
        let _res = handle(&mut deps, mock_env(), info, proposal_msg).unwrap();

        // proposal should be created.
        let state = config_read(&deps.storage).load().unwrap();
        assert_eq!(
            1,
            state.proposals.len(),
        );
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
        let _res = handle(&mut deps, mock_env(), info, proposal_msg).unwrap();

        // proposal should be created.
        let state = config_read(&deps.storage).load().unwrap();
        assert_eq!(
            1,
            state.proposals.len(),
        );
    }
}