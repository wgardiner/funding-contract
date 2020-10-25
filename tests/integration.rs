use cosmwasm_std::{coins, from_binary, HumanAddr, InitResponse};
use cosmwasm_vm::testing::{init, mock_env, mock_instance, query};
use cosmwasm_vm::Api;
use funding_contract::contract::calculate_distributions;
use funding_contract::msg::{InitMsg, QueryMsg, StateResponse};
use funding_contract::state::{Distribution, Proposal, Vote};

// This line will test the output of cargo wasm
static WASM: &[u8] =
    include_bytes!("../target/wasm32-unknown-unknown/release/funding_contract.wasm");
// You can uncomment this line instead to test productionified build from rust-optimizer
//static WASM: &[u8] = include_bytes!("../contract.wasm");

fn default_init_msg() -> InitMsg {
    let env = mock_env("owner", &coins(1000, "earth"));
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
    let mut deps = mock_instance(WASM, &[]);

    let msg = default_init_msg();
    let env = mock_env("owner", &coins(1000, "earth"));

    // we can just call .unwrap() to assert this was a success
    let res: InitResponse = init(&mut deps, env, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = query(
        &mut deps,
        //mock_env("owner", &coins(1000, "earth")),
        QueryMsg::GetState {},
    )
    .unwrap();
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
fn test_calculate_distributions() {
    let deps = mock_instance(WASM, &[]);
    let votes: Vec<Vote> = vec![
        Vote {
            voter: deps
                .api
                .canonical_address(&HumanAddr::from(&HumanAddr("voter_0".to_string())))
                .0
                .unwrap(),
            proposal: 0,
            amount: coins(1, "earth"),
        },
        Vote {
            voter: deps
                .api
                .canonical_address(&HumanAddr::from(&HumanAddr("voter_1".to_string())))
                .0
                .unwrap(),
            proposal: 0,
            amount: coins(4, "earth"),
        },
        Vote {
            voter: deps
                .api
                .canonical_address(&HumanAddr::from(&HumanAddr("voter_2".to_string())))
                .0
                .unwrap(),
            proposal: 1,
            amount: coins(9, "earth"),
        },
        Vote {
            voter: deps
                .api
                .canonical_address(&HumanAddr::from(&HumanAddr("voter_0".to_string())))
                .0
                .unwrap(),
            proposal: 1,
            amount: coins(16, "earth"),
        },
    ];
    let proposals = vec![
        Proposal {
            id: 0,
            name: "Proposal 0".to_string(),
            recipient: deps
                .api
                .canonical_address(&HumanAddr::from(&HumanAddr("recipient_0".to_string())))
                .0
                .unwrap(),
            description: "an okay proposal".to_string(),
            tags: "money".to_string(),
        },
        Proposal {
            id: 1,
            name: "Proposal 1".to_string(),
            recipient: deps
                .api
                .canonical_address(&HumanAddr::from(&HumanAddr("recipient_1".to_string())))
                .0
                .unwrap(),
            description: "an better proposal".to_string(),
            tags: "stuffed animals, parrots".to_string(),
        },
    ];
    let result: Vec<Distribution> = calculate_distributions(votes, proposals, coins(50, "shell"));
    // println!("{:#?}", result);
    assert_eq!(result.len(), 2);
    let distributions_for_prop_0: Vec<Distribution> = result
        .clone()
        .into_iter()
        .filter(|d| d.proposal == 0)
        .collect();
    let distributions_for_prop_1: Vec<Distribution> = result
        .clone()
        .into_iter()
        .filter(|d| d.proposal == 1)
        .collect();
    assert_eq!(
        distributions_for_prop_0[0].subsidy_actual.amount.u128(),
        7 as u128
    );
    assert_eq!(
        distributions_for_prop_1[0].subsidy_actual.amount.u128(),
        42 as u128
    );
    assert_eq!(
        distributions_for_prop_0[0]
            .distribution_actual
            .amount
            .u128(),
        12 as u128
    );
    assert_eq!(
        distributions_for_prop_1[0]
            .distribution_actual
            .amount
            .u128(),
        67 as u128
    );
}
