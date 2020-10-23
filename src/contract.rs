#![allow(clippy::too_many_arguments)]

use std::collections::HashMap;

use cosmwasm_std::{
    to_binary, Api, Binary, CanonicalAddr, Env, Extern, HandleResponse, HumanAddr, InitResponse,
    MessageInfo, Querier, StdResult, Storage, coin, Uint128
};

use crate::error::ContractError;
use crate::msg::{HandleMsg, InitMsg, ProposalListResponse, QueryMsg, StateResponse};
use crate::state::{config, config_read, Proposal, State, Vote, Distribution};

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
    let proposer_whitelist: Vec<_> = msg
        .proposer_whitelist
        .iter()
        .map(|x| deps.api.canonical_address(x))
        .filter_map(Result::ok)
        .collect();
    let voter_whitelist: Vec<_> = msg
        .voter_whitelist
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
        } => try_create_proposal(deps, env, info, state, recipient, name, description, tags),
        HandleMsg::CreateVote { proposal_id } => {
            try_create_vote(deps, env, info, state, proposal_id)
        }
    }
}

// TODO: Can we decrease the number of arguments?
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
    let sender_addr = deps.api.canonical_address(&info.sender)?;
    let recipient_addr = deps.api.canonical_address(&recipient)?;
    let sender_is_valid = validate_sender(sender_addr, state.proposer_whitelist);
    if !sender_is_valid {
        return Err(ContractError::Unauthorized {
            list_type: "proposer".to_string(),
        });
    }
    let period_is_valid = validate_period(
        env.block.time,
        state.proposal_period_start.unwrap(),
        state.proposal_period_end.unwrap(),
    );
    if !period_is_valid {
        return Err(ContractError::InvalidPeriod {
            period_type: "proposal".to_string(),
        });
    }
    if sender_is_valid && period_is_valid {
        config(&mut deps.storage).update(|mut state| -> Result<State, ContractError> {
            // state.count += 1;
            state.proposals.push(Proposal {
                id: state.proposals.len() as u32,
                name,
                description,
                tags,
                recipient: recipient_addr,
            });
            Ok(state)
        })?;
    }

    Ok(HandleResponse::default())
}

pub fn validate_period(time: u64, period_start: u64, period_end: u64) -> bool {
    if time < period_start {
        return false;
    }

    if time > period_end {
        return false;
    }

    true
}

pub fn validate_sender(addr: CanonicalAddr, list: Vec<CanonicalAddr>) -> bool {
    list.is_empty() || list.contains(&addr)
}

pub fn try_create_vote<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    info: MessageInfo,
    state: State,
    proposal_id: u32,
) -> Result<HandleResponse, ContractError> {
    let sender_is_valid = validate_sender(
        deps.api.canonical_address(&info.sender)?,
        state.voter_whitelist,
    );
    if !sender_is_valid {
        return Err(ContractError::Unauthorized {
            list_type: "voter".to_string(),
        });
    }
    let period_is_valid = validate_period(
        env.block.time,
        state.voting_period_start.unwrap(),
        state.voting_period_end.unwrap(),
    );
    if !period_is_valid {
        return Err(ContractError::InvalidPeriod {
            period_type: "voting".to_string(),
        });
    }
    let proposal_is_valid = state.proposals.len() as u32 >= proposal_id;
    if !proposal_is_valid {
        return Err(ContractError::InvalidProposal { id: proposal_id });
    }
    let voter = deps.api.canonical_address(&info.sender)?;
    if sender_is_valid && period_is_valid && proposal_is_valid {
        config(&mut deps.storage).update(|mut state| -> Result<State, ContractError> {
            state.votes.push(Vote {
                voter,
                proposal: proposal_id,
                amount: info.sent_funds,
            });
            Ok(state)
        })?;
    }

    Ok(HandleResponse::default())
}

pub fn get_unique_votes_by_voter(votes: &Vec<Vote>) -> Vec<Vote> {

    // let unique: Vec<_> = votes.into_iter()
    //     .map(|v| (v.proposal, v.amount[0].amount))
    //     .collect();
    // let mut unique: HashMap<_, _> = HashMap::new();
    // for

    // let hmap = votes.into_iter()
    //     .fold(
    //         HashMap::new(),
    //         |mut acc, x| {
    //             // *acc.entry(
    //             //     format!("{}--{}", x.voter, x.proposal.to_string())
    //             // ).or_insert(Uint128(0)) += x.amount[0].amount;
    //             // let (mut n, p, vaddr) = *acc.entry(
    //             let mut entry = *acc.entry(
    //                 format!("{}--{}", x.voter, x.proposal.to_string())
    //             // ).or_insert((Uint128(0), x.proposal, &x.voter));
    //             ).or_insert(Vote {
    //                 voter: x.voter,
    //                 proposal: x.proposal,
    //                 amount: x.amount,
    //             });

    //             // n += x.amount[0].amount;

    //             acc
    //         }
    //     );
    // let sum = votes.into_iter().fold(Uint128(0), |acc, x|  x.amount[0].amount + acc);

    // let mut unique: HashMap = HashMap::new();

    let mut unique: HashMap<String, Vote> = HashMap::new();
    for vote in votes {
        let tag = format!("{}--{}", vote.voter, vote.proposal.to_string());
        // println!("{}", tag);

        if !unique.contains_key(&tag) {
            unique.insert(tag, vote.clone());
        } else {
            // let tag_entry = unique.entry(tag);
            let value = unique.get(&tag).unwrap();
            // println!("{:?}", value);
            // println!("{}", 0);
            let new_entry = Vote{
                voter: vote.voter.clone(),
                proposal: vote.proposal,
                amount: vec![coin(
                    vote.amount[0].amount.u128() + value.amount[0].amount.u128(),
                    &vote.amount[0].denom
                )],
            };
            unique.insert(tag, new_entry);
            // tag_entry.value.amount = vec![coin(tag_entry.amount[0].amount + vote.amount[0].amount, vote.amount[0].denom)];
            // tag_entry = Vote {}
        }
    }

    println!("{:?}", unique);
    // println!("{}", sum);

    // votes.clone()
    unique.values().cloned().collect()


}

pub fn calculate_distribution(_votes: Vec<Vote>, _proposals: Vec<Proposal>) -> Vec<Distribution> {
    vec![
        Distribution {
            proposal: 0,
            votes: vec![coin(1, "shell")],
            distribution_ideal: coin(1, "shell"),
            subsidy_ideal: coin(1, "shell"),
            distribution_actual: coin(1, "shell"),
            subsidy_actual: coin(1, "shell"),
        }
    ]
}

// TODO: Add query Proposal + Votes by Proposal ID.
pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    _env: Env,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetState {} => to_binary(&query_state(deps)?),
        QueryMsg::ProposalList {} => to_binary(&query_proposal_list(deps)?),
    }
}

fn query_state<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<StateResponse> {
    let state = config_read(&deps.storage).load()?;
    let proposer_whitelist = state
        .proposer_whitelist
        .iter()
        .map(|x| deps.api.human_address(x))
        .filter_map(Result::ok)
        .collect();
    let voter_whitelist = state
        .voter_whitelist
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

fn query_proposal_list<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<ProposalListResponse> {
    let state = config_read(&deps.storage).load()?;
    let proposals = state.proposals;
    Ok(ProposalListResponse { proposals })
}
