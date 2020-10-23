use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use funding_contract::msg::{
    CheckDistributionsResponse, CreateProposalResponse, HandleMsg, InitMsg, ProposalListResponse,
    ProposalStateResponse, QueryMsg, StateResponse,
};
use funding_contract::state::State;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InitMsg), &out_dir);
    export_schema(&schema_for!(HandleMsg), &out_dir);
    export_schema(&schema_for!(CreateProposalResponse), &out_dir);
    export_schema(&schema_for!(CheckDistributionsResponse), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(ProposalListResponse), &out_dir);
    export_schema(&schema_for!(ProposalStateResponse), &out_dir);
    export_schema(&schema_for!(State), &out_dir);
    export_schema(&schema_for!(StateResponse), &out_dir);
}
