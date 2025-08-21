use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum ExecuteMsg {
    PostJob { 
        description: String,
        deadline: Option<u64>,
    },
    AcceptJob { job_id: u64 },
    SubmitProof { job_id: u64, proof: String },
    AcceptProof { job_id: u64 },
    RejectProof { job_id: u64 },
    CancelJob { job_id: u64 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum QueryMsg {
    GetJob { job_id: u64 },
    ListJobs {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct JobResponse {
    pub job: crate::state::Job,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct JobsResponse {
    pub jobs: Vec<crate::state::Job>,
}
