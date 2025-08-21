use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum JobStatus {
    Open,
    InProgress,
    Completed,
    Rejected,
    Cancelled,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Job {
    pub id: u64,
    pub client: Addr,
    pub worker: Option<Addr>,
    pub description: String,
    pub proof: Option<String>,
    pub accepted: bool,
    pub escrow_amount: Coin,
    pub status: JobStatus,
    pub deadline: Option<u64>,
}

pub const STATE: Item<State> = Item::new("state");
pub const JOB_ID: Item<u64> = Item::new("job_id");
pub const JOBS: Map<u64, Job> = Map::new("jobs");
