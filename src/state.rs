use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, Coin, Storage};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};

pub static CONFIG_KEY: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    // pub count: i32,
    pub owner: CanonicalAddr,
    // funding pool is stored in contract balance?
    // pub balance: Vec<Coin>,
    pub name: String,
    pub proposer_whitelist: Vec<CanonicalAddr>,
    pub voter_whitelist: Vec<CanonicalAddr>,
    // // pub voting_min_period: u32, // in seconds
    // // pub proposal_min_period: u32,
    // // pub min_voting_period: u32,
    // // pub min_proposal_period: u32,
    pub proposal_period_start: Option<u64>, // Option values are optional
    pub proposal_period_end: Option<u64>,
    pub voting_period_start: Option<u64>,
    pub voting_period_end: Option<u64>,
    // pub funding_formula: Option<String>,
    pub proposals: Vec<Proposal>,
    pub votes: Vec<Vote>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Proposal {
    pub id: u32,
    pub name: String,
    pub recipient: CanonicalAddr,
    pub description: String,
    pub tags: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Vote {
    pub voter: CanonicalAddr,
    pub proposal: u32, // reference to proposal id
    // pub txid, // would this be valuable?
    // pub amount: u32, // can this just be referenced from the contract's trasaction history?
    pub amount: Vec<Coin>, // can this just be referenced from the contract's trasaction history?
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Distribution {
    pub proposal: u32, // reference to proposal id
    pub votes: Vec<Coin>,
    pub distribution_ideal: Coin,
    pub subsidy_ideal: Coin,
    pub distribution_actual: Coin,
    pub subsidy_actual: Coin,
}

pub fn config<S: Storage>(storage: &mut S) -> Singleton<S, State> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, State> {
    singleton_read(storage, CONFIG_KEY)
}
