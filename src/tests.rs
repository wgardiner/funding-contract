#[cfg(test)]
mod tests {
    use crate::contract::{init, handle, query};
    use crate::msg::{StateResponse, HandleMsg, InitMsg, QueryMsg};
    use crate::state::{config_read};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MockStorage, MockApi, MockQuerier};
    use cosmwasm_std::{ Extern, HumanAddr, coins, from_binary};

    fn default_init_msg() -> InitMsg {
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
            // proposal_period_start: None,
            // proposal_period_end: None,
            // voting_period_start: None,
            // voting_period_end: None,
            proposal_period_start: Some(1602896282),
            proposal_period_end: Some(1602896282),
            voting_period_start: Some(1602896282),
            voting_period_end: Some(1602896282),
        }
    }

    fn mock_init(mut deps: &mut Extern<MockStorage, MockApi, MockQuerier>) {
        let msg = default_init_msg();
        let info = mock_info("creator", &coins(1000, "earth"));
        let _res = init(&mut deps, mock_env(), info, msg).unwrap();
    }

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = InitMsg {
            // count: 17
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
            // proposal_period_start: None,
            // proposal_period_end: None,
            // voting_period_start: None,
            // voting_period_end: None,
            proposal_period_start: Some(1602896282),
            proposal_period_end: Some(1602896282),
            voting_period_start: Some(1602896282),
            voting_period_end: Some(1602896282),
        };
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

    }

    #[test]
    fn fails_create_proposal_invalid_period() {

    }

    #[test]
    fn fails_create_proposal_insufficient_data() {

    }

    #[test]
    fn create_proposal() {
        let mut deps = mock_dependencies(&[]);
        mock_init(&mut deps);

        // Create proposal.
        let proposal_msg = HandleMsg::CreateProposal {
            name: "My proposal".to_string(),
            recipient: HumanAddr::from("proposal_recipient"),
            description: "The proposal description".to_string(),
            tags: "one two three".to_string(),
        };

        let info = mock_info("creator", &coins(1000, "earth"));
        let res = handle(&mut deps, mock_env(), info, proposal_msg).unwrap();

        // proposal should be created.
        let state = config_read(&deps.storage).load().unwrap();
        assert_eq!(
            1,
            state.proposals.len(),
        );
    }
}