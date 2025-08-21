use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: Addr,
    pub next_job_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Job {
    pub id: u64,
    pub client: Addr,
    pub description: String,
    pub escrow_amount: Coin,
    pub worker: Option<Addr>,
    pub proof: Option<String>,
    pub status: JobStatus,
    pub created_at: u64,
    pub deadline: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum JobStatus {
    Open,
    InProgress,
    Completed,
    Rejected,
    Cancelled,
}

pub const STATE: Item<State> = Item::new("state");
pub const JOBS: Map<u64, Job> = Map::new("jobs");
