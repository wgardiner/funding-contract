use cosmwasm_std::{
    to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, MessageInfo, Querier,
    StdResult, Storage, CanonicalAddr, HumanAddr
};

use crate::error::ContractError;
use crate::msg::{StateResponse, HandleMsg, InitMsg, QueryMsg, CreateProposal};
use crate::state::{config, config_read, State, Vote, Proposal};

// pub fn mapHumanToCanonicalAddr(list: Vec<_>) -> Vec<CanonicalAddr> {
//     list
//         .iter()
//         .map(|x| deps.api.canonical_address(x))
//         .filter_map(Result::ok)
//         .collect()
// }

// Note, you can use StdResult in some functions where you do not
// make use of the custom errors
pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    info: MessageInfo,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    // let pw: Vec<CanonicalAddr> = msg.proposer_whitelist.into_iter().map(|x| deps.api.canonical_address(&x)).collect::<Vec<CanonicalAddr>>()?;
    // TODO: this should probably just fail if the user attempts to instantiate the contract
    // with an address that can't be converted to cannonical form in the whitelists.
    // Currently it'll just filter out any addresses that fail conversion.
    let proposer_whitelist: Vec<_> = msg.proposer_whitelist
        .iter()
        .map(|x| deps.api.canonical_address(x))
        .filter_map(Result::ok)
        .collect();
    let voter_whitelist: Vec<_> = msg.voter_whitelist
        .iter()
        .map(|x| deps.api.canonical_address(x))
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    let state = State {
        // count: msg.count,
        name: msg.name,
        owner: deps.api.canonical_address(&info.sender)?,
        proposer_whitelist,
        voter_whitelist,
        // // proposal_min_period: 10,
        // // voting_min_period: 10,
        proposal_period_start: msg.proposal_period_start,
        proposal_period_end: msg.proposal_period_end,
        voting_period_start: msg.voting_period_start,
        voting_period_end: msg.voting_period_end,
        // funding_formula: Some("QUADRATIC".to_string()),
        votes: Vec::new(),
        proposals: Vec::new(),
    };
    config(&mut deps.storage).save(&state)?;

    Ok(InitResponse::default())
    // TODO: handle expired with Err
}

// And declare a custom Error variant for the ones where you will want to make use of it
pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    info: MessageInfo,
    msg: HandleMsg,
) -> Result<HandleResponse, ContractError> {
    let state = config_read(&deps.storage).load()?;
    match msg {
        HandleMsg::CreateProposal {
            name,
            description,
            recipient,
            tags,
        } => try_create_proposal(
            deps,
            env,
            info,
            state,
            recipient,
            name,
            description,
            tags,
        ),
        HandleMsg::CreateVote { proposal_id } => try_create_vote(deps, env, info, state, proposal_id),
        // HandleMsg::Increment {} => try_increment(deps),
        // HandleMsg::Reset { count } => try_reset(deps, info, count),
    }
}

pub fn try_create_proposal<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    info: MessageInfo,
    state: State,
    // msg: CreateProposal,
    recipient: HumanAddr,
    name: String,
    description: String,
    tags: String,
) -> Result<HandleResponse, ContractError> {
    // TODO: check if sender matches whitelist
    let senderAddr = deps.api.canonical_address(&info.sender)?;
    let recipientAddr = deps.api.canonical_address(&recipient)?;
    let sender_is_valid = validate_sender(senderAddr, state.proposer_whitelist);
    let period_is_valid = validate_period(
        env.block.time,
        state.proposal_period_start.unwrap(),
        state.proposal_period_end.unwrap()
    );
    if sender_is_valid && period_is_valid {
        config(&mut deps.storage).update(|mut state| -> Result<State, ContractError> {
            // state.count += 1;
            state.proposals.push(Proposal {
                id: state.proposals.len() as u32,
                name,
                description,
                tags,
                recipient: recipientAddr,
            });
            Ok(state)
        })?;
    }


    Ok(HandleResponse::default())
}

pub fn validate_period(time: u64, period_start: u64, period_end: u64) -> bool {
    true
}

pub fn validate_sender(addr: CanonicalAddr, list: Vec<CanonicalAddr>) -> bool {
    // if deps.api.canonical_address(&env.message.sender)? != state.arbiter {
    //     Err(StdError::unauthorized())
    // } else if state.is_expired(&env) {
    //     Err(StdError::generic_err("escrow expired"))
    // } else {
        true
    // }
}

pub fn try_create_vote<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    info: MessageInfo,
    state: State,
    proposal_id: u32,
) -> Result<HandleResponse, ContractError> {
    // TODO: check if sender matches whitelist
    let sender_is_valid = validate_sender(deps.api.canonical_address(&info.sender)?, state.voter_whitelist);
    let period_is_valid = validate_period(
        env.block.time,
        state.voting_period_start.unwrap(),
        state.voting_period_end.unwrap(),
    );
    let voter = deps.api.canonical_address(&info.sender)?;
    if sender_is_valid && period_is_valid {
        config(&mut deps.storage).update(|mut state| -> Result<State, ContractError> {
            state.votes.push(Vote {
                voter: voter,
                proposal: proposal_id,
                amount: info.sent_funds,
            });
            Ok(state)
        })?;
    }

    Ok(HandleResponse::default())
}

// pub fn try_increment<S: Storage, A: Api, Q: Querier>(
//     deps: &mut Extern<S, A, Q>,
// ) -> Result<HandleResponse, ContractError> {
//     config(&mut deps.storage).update(|mut state| -> Result<_, ContractError> {
//         state.count += 1;
//         Ok(state)
//     })?;

//     Ok(HandleResponse::default())
// }

// pub fn try_reset<S: Storage, A: Api, Q: Querier>(
//     deps: &mut Extern<S, A, Q>,
//     info: MessageInfo,
//     count: i32,
// ) -> Result<HandleResponse, ContractError> {
//     let api = &deps.api;
//     config(&mut deps.storage).update(|mut state| -> Result<_, ContractError> {
//         if api.canonical_address(&info.sender)? != state.owner {
//             return Err(ContractError::Unauthorized {});
//         }
//         state.count = count;
//         Ok(state)
//     })?;
//     Ok(HandleResponse::default())
// }

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    _env: Env,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetState {} => to_binary(&query_state(deps)?),
    }
}

fn query_state<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<StateResponse> {
    let state = config_read(&deps.storage).load()?;
    let proposer_whitelist = state.proposer_whitelist
        .iter()
        .map(|x| deps.api.human_address(x))
        .filter_map(Result::ok)
        .collect();
    let voter_whitelist = state.voter_whitelist
        .iter()
        .map(|x| deps.api.human_address(x))
        .filter_map(Result::ok)
        .collect();
    Ok(StateResponse {
        name: state.name,
        proposer_whitelist,
        voter_whitelist,
        proposal_period_start: state.proposal_period_start,
        proposal_period_end: state.proposal_period_end,
        voting_period_start: state.voting_period_start,
        voting_period_end: state.voting_period_end,
    })
}

// pub fn query<S: Storage, A: Api, Q: Querier>(
//     deps: &Extern<S, A, Q>,
//     _env: Env,
//     msg: QueryMsg,
// ) -> StdResult<Binary> {
//     match msg {
//         QueryMsg::GetCount {} => to_binary(&query_count(deps)?),
//     }
// }

// fn query_count<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<CountResponse> {
//     let state = config_read(&deps.storage).load()?;
//     Ok(CountResponse { count: state.count })
// }

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MockStorage, MockApi, MockQuerier};
    use cosmwasm_std::{coins, from_binary, HumanAddr};

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

    // #[test]
    // fn increment() {
    //     let mut deps = mock_dependencies(&coins(2, "token"));

    //     let msg = InitMsg { count: 17 };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = init(&mut deps, mock_env(), info, msg).unwrap();

    //     // beneficiary can release it
    //     let info = mock_info("anyone", &coins(2, "token"));
    //     let msg = HandleMsg::Increment {};
    //     let _res = handle(&mut deps, mock_env(), info, msg).unwrap();

    //     // should increase counter by 1
    //     let res = query(&deps, mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(18, value.count);
    // }

    // #[test]
    // fn reset() {
    //     let mut deps = mock_dependencies(&coins(2, "token"));

    //     let msg = InitMsg { count: 17 };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = init(&mut deps, mock_env(), info, msg).unwrap();

    //     // beneficiary can release it
    //     let unauth_info = mock_info("anyone", &coins(2, "token"));
    //     let msg = HandleMsg::Reset { count: 5 };
    //     let res = handle(&mut deps, mock_env(), unauth_info, msg);
    //     match res {
    //         Err(ContractError::Unauthorized {}) => {}
    //         _ => panic!("Must return unauthorized error"),
    //     }

    //     // only the original creator can reset the counter
    //     let auth_info = mock_info("creator", &coins(2, "token"));
    //     let msg = HandleMsg::Reset { count: 5 };
    //     let _res = handle(&mut deps, mock_env(), auth_info, msg).unwrap();

    //     // should now be 5
    //     let res = query(&deps, mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(5, value.count);
    // }
}
