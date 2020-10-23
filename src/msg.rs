use crate::state::{Distribution, Proposal, Vote};
use cosmwasm_std::HumanAddr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    // pub count: i32,
    // Coins for funding pool are attached to TX
    pub name: String,
    pub proposer_whitelist: Vec<HumanAddr>,
    pub voter_whitelist: Vec<HumanAddr>,
    // pub proposal_min_period: Option<u32>,
    // pub voting_min_period: Option<u32>,
    pub proposal_period_start: Option<u64>,
    pub proposal_period_end: Option<u64>,
    pub voting_period_start: Option<u64>,
    pub voting_period_end: Option<u64>,
    // pub funding_formula: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    // Increment {},
    // Reset { count: i32 },
    // Create(CreateProposal),
    CreateProposal {
        name: String,
        recipient: HumanAddr,
        description: String,
        tags: String,
    },
    CreateVote {
        proposal_id: u32,
    },
    CheckDistributions {},
    DistributeFunds {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CreateProposal {
    pub name: String,
    pub recipient: HumanAddr,
    pub description: String,
    pub tags: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CreateProposalResponse {
    pub proposal_id: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CheckDistributionsResponse {
    pub distributions: Vec<Distribution>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetState {},
    ProposalList {},
    ProposalState { proposal_id: u32 },
}

// // We define a custom struct for each query response
// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// pub struct CountResponse {
//     pub count: i32,
// }
// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    // pub count: Vec<CanonicalAddr>,
    pub name: String,
    pub proposer_whitelist: Vec<HumanAddr>,
    pub voter_whitelist: Vec<HumanAddr>,
    pub proposal_period_start: Option<u64>,
    pub proposal_period_end: Option<u64>,
    pub voting_period_start: Option<u64>,
    pub voting_period_end: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ProposalListResponse {
    pub proposals: Vec<Proposal>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ProposalStateResponse {
    pub proposal: Proposal,
    pub votes: Vec<Vote>,
}
