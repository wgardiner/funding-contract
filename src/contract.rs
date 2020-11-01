#![allow(clippy::too_many_arguments)]

use std::collections::HashMap;

use integer_sqrt::IntegerSquareRoot;

use cosmwasm_std::{
    attr, coin, to_binary, Api, BankMsg, Binary, CanonicalAddr, Coin, CosmosMsg, Env, Extern,
    HandleResponse, HumanAddr, InitResponse, MessageInfo, Querier, StdError, StdResult, Storage,
};

use crate::error::ContractError;
use crate::msg::{
    CheckDistributionsResponse, CreateProposalResponse, HandleMsg, InitMsg, ProposalListResponse,
    ProposalStateResponse, QueryMsg, StateResponse,
};
use crate::state::{config, config_read, Distribution, Proposal, State, Vote};

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
        HandleMsg::CheckDistributions {} => try_check_distributions(deps, env, info, state),
        HandleMsg::DistributeFunds {} => try_distribute_funds(deps, env, info, state),
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
    let proposal_id = state.proposals.len() as u32;
    if sender_is_valid && period_is_valid {
        config(&mut deps.storage).update(|mut state| -> Result<State, ContractError> {
            // state.count += 1;
            state.proposals.push(Proposal {
                id: proposal_id,
                name,
                description,
                tags,
                recipient: recipient_addr,
            });
            Ok(state)
        })?;
    }

    let res = HandleResponse {
        messages: vec![],
        attributes: vec![],
        data: Some(to_binary(&CreateProposalResponse { proposal_id })?),
    };
    Ok(res)
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
    let proposal_is_valid = state.proposals.len() as u32 > proposal_id;
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

pub fn try_check_distributions<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    _info: MessageInfo,
    state: State,
) -> Result<HandleResponse, ContractError> {
    // Distributions can only be checked after proposal period.
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

    let distributions: Vec<Distribution> = calculate_distributions(
        state.votes,
        state.proposals,
        deps.querier.query_all_balances(&env.contract.address)?,
        // vec![coin(100_000, "ucosm")],
    );

    let res = HandleResponse {
        messages: vec![],
        attributes: vec![
            attr(
                "distributions",
                to_binary(&CheckDistributionsResponse {
                    distributions: distributions.clone(),
                })?,
            ),
            attr(
                "balance",
                format!(
                    "{:?}",
                    deps.querier.query_all_balances(env.contract.address)?
                ),
            ),
        ],
        data: Some(to_binary(&CheckDistributionsResponse { distributions })?),
    };
    Ok(res)
}

pub fn try_distribute_funds<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    info: MessageInfo,
    state: State,
) -> Result<HandleResponse, ContractError> {
    // Only the contract owner can distribute funds.
    let sender_is_valid =
        validate_sender(deps.api.canonical_address(&info.sender)?, vec![state.owner]);
    if !sender_is_valid {
        return Err(ContractError::Unauthorized {
            list_type: "owner".to_string(),
        });
    }
    // Distributions can only be checked after proposal period.
    let period_is_valid =
        // validate_period(env.block.time, state.voting_period_end.unwrap(), u64::MAX);
        true;
    if !period_is_valid {
        return Err(ContractError::InvalidPeriod {
            period_type: "voting".to_string(),
        });
    }

    let distributions: Vec<Distribution> = calculate_distributions(
        state.votes,
        state.proposals,
        deps.querier.query_all_balances(&env.contract.address)?,
    );

    // TODO: Send funds to proposal recipients.
    send_distributions(deps, env, distributions, &"distribute funds".to_string())

    // TODO: Finalize response data.
    // Should this return the same Vec<Distribution> data as CheckDistributions?
    // let res = HandleResponse {
    //     messages: vec![],
    //     attributes: vec![],
    //     data: Some(to_binary(&CheckDistributionsResponse { distributions })?),
    // };
    // Ok(res)
}

pub fn is_coin_micro(denom: &str) -> bool {
    denom.starts_with('u')
}

pub fn get_normalized_votes(votes: &[Vote]) -> Vec<Vote> {
    let mut unique: HashMap<String, Vote> = HashMap::new();
    for vote in votes {
        let tag = format!("{}--{}", vote.voter, vote.proposal.to_string());

        let denom = &vote.amount[0].denom;
        let mut new_denom = denom.clone();
        let mut math_factor = 1u128;
        if !is_coin_micro(denom) {
            new_denom = format!("{}{}", "u", denom.to_string());
            math_factor *= 1_000_000u128;
        }

        let norm_vote = Vote {
            voter: vote.voter.clone(),
            proposal: vote.proposal,
            amount: vec![coin(vote.amount[0].amount.u128() * math_factor, &new_denom)],
        };

        // by default add the vote itself.
        let new_entry = if !unique.contains_key(&tag) {
            norm_vote
        } else {
            // if the tag exists, update the existing norm_vote amount.
            let value = unique.get(&tag).unwrap();

            Vote {
                voter: norm_vote.voter.clone(),
                proposal: norm_vote.proposal,
                amount: vec![coin(
                    norm_vote.amount[0].amount.u128() + value.amount[0].amount.u128(),
                    &norm_vote.amount[0].denom,
                )],
            }
        };

        // add new key, or update existing.
        unique.insert(tag, new_entry);
    }
    unique.values().cloned().collect()
}

pub fn calculate_distributions(
    votes: Vec<Vote>,
    proposals: Vec<Proposal>,
    budget_contstraint: Vec<Coin>,
) -> Vec<Distribution> {
    let denom = &budget_contstraint[0].denom;
    let mut new_denom = denom.clone();
    let math_factor = 1_000_000u128;

    // let budget_value = budget_contstraint[0].amount.u128() as u128 * math_factor;
    let mut budget_value = budget_contstraint[0].amount.u128() as u128;
    // Multiply values so that we don't have to convert to floats

    if !is_coin_micro(&budget_contstraint[0].denom) {
        new_denom = format!("{}{}", "u".to_string(), denom.to_string());
        budget_value *= math_factor;

        // math_factor *= math_factor;
    }

    // Collapse multiple votes all votes by a single voter for a single proposal
    let unique_votes = get_normalized_votes(&votes);

    // TODO: convert to same currency? normalize to shell or ushell

    struct DistIdeal {
        proposal: u32,
        recipient: CanonicalAddr,
        // votes: Vec<f64>,
        // distribution_ideal: f64,
        // subsidy_ideal: f64
        votes: Vec<u128>,
        distribution_ideal: u128,
        subsidy_ideal: u128,
    }

    let ideal_results: Vec<_> = proposals
        .into_iter()
        .map(|p| {
            // Convert votes to a nicer format
            // let proposal_votes: Vec<f64> = unique_votes
            let proposal_votes: Vec<u128> = unique_votes
                .iter()
                .filter(|v| v.proposal == p.id)
                // .map(|v| v.amount[0].amount.u128() as f64)
                // .map(|v| v.amount[0].amount.u128() as u128 * math_factor)
                .map(|v| v.amount[0].amount.u128() as u128)
                .collect();

            // let distribution_ideal: f64 = proposal_votes.iter().map(|v| v.sqrt()).sum::<f64>().powi(2);
            // let subsidy_ideal: f64 = distribution_ideal - proposal_votes.iter().sum::<f64>();
            let distribution_ideal: u128 = proposal_votes
                .iter()
                .map(|v| v.integer_sqrt())
                .sum::<u128>()
                .pow(2);

            let total_votes: u128 = proposal_votes.iter().sum();
            let subsidy_ideal: u128 = match distribution_ideal > total_votes {
                true => distribution_ideal - total_votes,
                false => 0,
            };

            DistIdeal {
                proposal: p.id,
                recipient: p.recipient,
                votes: proposal_votes,
                distribution_ideal,
                subsidy_ideal,
            }
        })
        .collect();

    // let constraint_factor: f64 = ideal_results.iter().map(|x| x.subsidy_ideal).sum::<f64>() / budget_value;
    let constraint_factor: u128 =
        math_factor * ideal_results.iter().map(|x| x.subsidy_ideal).sum::<u128>() / budget_value;

    ideal_results
        .into_iter()
        .map(|p| {
            // let total_votes: f64 = p.votes.iter().sum();
            // let distribution_actual: f64 = (p.distribution_ideal - total_votes) / constraint_factor + total_votes;
            // let subsidy_actual: f64 = distribution_actual - total_votes;
            let total_votes: u128 = p.votes.iter().sum();
            // let distribution_actual: u128 = math_factor * (p.distribution_ideal - total_votes)
            let scary_term: u128 = match p.distribution_ideal > total_votes {
                true => p.distribution_ideal - total_votes,
                false => 0,
            };
            let distribution_actual: u128 =
                math_factor * scary_term / constraint_factor + total_votes;
            let subsidy_actual: u128 = distribution_actual - total_votes;
            Distribution {
                proposal: p.proposal,
                recipient: p.recipient,
                votes: p
                    .votes
                    .iter()
                    // .map(|v| coin((*v * math_factor) as u128, &new_denom))
                    .map(|v| coin((*v) as u128, &new_denom))
                    .collect(),
                // distribution_ideal: coin((p.distribution_ideal * math_factor) as u128, &new_denom),
                // subsidy_ideal: coin((p.subsidy_ideal * math_factor) as u128, &new_denom),
                // distribution_actual: coin((distribution_actual / math_factor) as u128, &new_denom),
                // subsidy_actual: coin((subsidy_actual / math_factor) as u128, &new_denom),
                distribution_ideal: coin((p.distribution_ideal) as u128, &new_denom),
                subsidy_ideal: coin((p.subsidy_ideal) as u128, &new_denom),
                distribution_actual: coin((distribution_actual) as u128, &new_denom),
                subsidy_actual: coin((subsidy_actual) as u128, &new_denom),
            }
        })
        .collect()

    // vec![Distribution{
    //     proposal: s,
    //     votes: vec![coin(1, "SHELL")],
    //     distribution_ideal: coin(1, "SHELL"),
    //     distribution_actual: coin(1, "SHELL"),
    //     subsidy_ideal: coin(1, "SHELL"),
    //     subsidy_actual: coin(1, "SHELL"),
    // }]
}

fn send_distributions<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    distributions: Vec<Distribution>,
    action: &str,
) -> Result<HandleResponse, ContractError> {
    let attributes = vec![attr("action", action)];

    // it should cost ~800 ucosm to send a
    let send_cost = 1000;
    let messages = distributions
        .into_iter()
        .filter_map(|d| {
            let contract_address = env.contract.address.clone();
            let recipient = deps.api.human_address(&d.recipient).unwrap();
            let amount_send = match d.distribution_actual.amount.u128() > send_cost {
                true => d.distribution_actual.amount.u128() - send_cost,
                false => 0,
            };
            if amount_send > 0 {
                Some(CosmosMsg::Bank(BankMsg::Send {
                    from_address: contract_address,
                    to_address: recipient,
                    // amount: vec![d.distribution_actual],
                    amount: vec![coin(amount_send, "ucosm")],
                    // amount: vec![coin(10, "ucosm")],
                }))
            } else {
                None
            }
        })
        .collect();

    let r = HandleResponse {
        messages,
        data: None,
        attributes,
    };
    Ok(r)
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
        QueryMsg::ProposalState { proposal_id } => query_proposal_state(deps, proposal_id),
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

fn query_proposal_state<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    proposal_id: u32,
) -> StdResult<Binary> {
    let state = config_read(&deps.storage).load()?;
    let proposal = match state.proposals.into_iter().find(|p| p.id == proposal_id) {
        Some(proposal) => Some(proposal),
        None => return Err(StdError::generic_err("Proposal does not exist")),
    }
    .unwrap();

    let votes: Vec<Vote> = state
        .votes
        .into_iter()
        .filter(|v| v.proposal == proposal_id)
        .collect();
    let resp = ProposalStateResponse { proposal, votes };
    to_binary(&resp)
}
