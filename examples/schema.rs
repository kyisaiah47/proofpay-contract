use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use proof_of_work_contract::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, JobResponse, JobsResponse};
use proof_of_work_contract::state::{Job, JobStatus, State};
use std::env::current_dir;
use std::fs::create_dir_all;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(JobResponse), &out_dir);
    export_schema(&schema_for!(JobsResponse), &out_dir);
    export_schema(&schema_for!(Job), &out_dir);
    export_schema(&schema_for!(JobStatus), &out_dir);
    export_schema(&schema_for!(State), &out_dir);
}
