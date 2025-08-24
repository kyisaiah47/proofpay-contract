use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: Addr,
    pub next_payment_id: u64,
    pub next_task_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct User {
    pub wallet_address: Addr,
    pub username: String,
    pub display_name: String,
    pub profile_picture: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Friendship {
    pub user1: String, // username
    pub user2: String, // username
    pub created_at: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FriendRequest {
    pub from_username: String,
    pub to_username: String,
    pub status: FriendRequestStatus,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum FriendRequestStatus {
    Pending,
    Accepted,
    Declined,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Payment {
    pub id: u64,
    pub from_username: String,
    pub to_username: String,
    pub amount: Coin,
    pub description: String,
    pub payment_type: PaymentType,
    pub proof_type: ProofType,
    pub proof_data: Option<String>,
    pub status: PaymentStatus,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum PaymentType {
    DirectPayment,    // Immediate payment
    PaymentRequest,   // Request money owed
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum PaymentStatus {
    Pending,          // Waiting for action
    ProofSubmitted,   // Proof submitted, waiting approval
    Completed,        // Payment completed
    Rejected,         // Payment rejected
    Cancelled,        // Payment cancelled
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum ProofType {
    None,             // No proof required
    Photo,            // Photo proof
    Document,         // Document proof
    Location,         // Location proof
    ZkTLS,           // zkTLS verification
    Manual,          // Manual verification
    Soft,            // Task: no escrow, payer approves manually
    Hybrid,          // Task: escrowed, zkTLS proof + dispute window
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum TaskStatus {
    Escrowed,         // Funds held in escrow
    ProofSubmitted,   // Proof submitted, waiting for processing
    PendingRelease,   // Hybrid mode: waiting for dispute window to expire
    Released,         // Task completed, payment sent
    Disputed,         // Task under dispute
    Refunded,         // Task expired/cancelled, funds returned
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Task {
    pub id: u64,
    pub payer: String,           // username
    pub worker: String,          // username
    pub amount: cosmwasm_std::Coin,
    pub proof_type: ProofType,
    pub status: TaskStatus,
    pub deadline_ts: u64,        // Unix timestamp when task expires
    pub review_window_secs: Option<u64>, // For hybrid mode dispute window
    pub endpoint: String,        // API endpoint for zkTLS verification
    pub evidence_hash: Option<String>,   // Hash of evidence for soft mode
    pub zk_proof_hash: Option<String>,   // Hash of zkTLS proof
    pub verified_at: Option<u64>,        // When proof was verified
    pub verifier_id: Option<String>,     // ID of verifier (if any)
    pub description: String,
    pub created_at: u64,
    pub updated_at: u64,
}

// Storage Maps
pub const STATE: Item<State> = Item::new("state");

// User Management
pub const USERS_BY_USERNAME: Map<String, User> = Map::new("users_by_username");
pub const USERS_BY_WALLET: Map<Addr, String> = Map::new("users_by_wallet"); // wallet -> username

// Friends System
pub const FRIENDSHIPS: Map<(String, String), Friendship> = Map::new("friendships");
pub const FRIEND_REQUESTS: Map<(String, String), FriendRequest> = Map::new("friend_requests");

// Payment System
pub const PAYMENTS: Map<u64, Payment> = Map::new("payments");
pub const USER_PAYMENTS: Map<(String, u64), bool> = Map::new("user_payments"); // (username, payment_id) -> exists

// Task System
pub const TASKS: Map<u64, Task> = Map::new("tasks");
pub const USER_TASKS: Map<(String, u64), bool> = Map::new("user_tasks"); // (username, task_id) -> exists
