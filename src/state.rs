use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: Addr,
    pub next_payment_id: u64,
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
    HelpRequest,      // Request help/work
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
