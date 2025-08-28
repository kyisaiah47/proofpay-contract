use cosmwasm_std::{Binary, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    RegisterUser {
        username: String,
    },
    AddAuthorizedAddress {
        address: String,
    },
    RemoveAuthorizedAddress {
        address: String,
    },
    CreatePayment {
        recipient: String,
        amount: Uint128,
        token: Option<String>,
        proof_type: Option<ProofType>,
        description: Option<String>,
        requires_proof: bool,
    },
    SubmitProof {
        payment_id: String,
        proof: Binary,
    },
    CompletePayment {
        payment_id: String,
    },
    DisputePayment {
        payment_id: String,
        reason: String,
    },
    CancelPayment {
        payment_id: String,
    },
    SendIbcPayment {
        channel: String,
        recipient: String,
        amount: Uint128,
        token: Option<String>,
        proof_data: Option<Binary>,
        description: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetUser { address: String },
    GetUserByUsername { username: String },
    IsUsernameAvailable { username: String },
    IsAuthorized { user: String, address: String },
    GetPayment { payment_id: String },
    GetUserPayments { user: String },
    GetStats {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserResponse {
    pub username: String,
    pub is_registered: bool,
    pub created_at: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PaymentResponse {
    pub id: String,
    pub sender: String,
    pub recipient: String,
    pub amount: Uint128,
    pub token: Option<String>,
    pub status: PaymentStatus,
    pub proof_type: Option<ProofType>,
    pub proof_data: Option<Binary>,
    pub description: Option<String>,
    pub created_at: u64,
    pub completed_at: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StatsResponse {
    pub total_users: u64,
    pub total_payments: u64,
    pub total_volume: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    Pending,
    Completed,
    Disputed,
    Cancelled,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProofType {
    Text,
    Photo,
    ZkTLS,
    Hybrid,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CrossChainPaymentPacket {
    pub sender: String,
    pub recipient: String,
    pub amount: Uint128,
    pub token: Option<String>,
    pub proof_data: Option<Binary>,
    pub description: Option<String>,
}