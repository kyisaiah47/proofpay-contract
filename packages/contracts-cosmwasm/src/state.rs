use cosmwasm_std::{Binary, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::msg::{PaymentStatus, ProofType};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct User {
    pub username: String,
    pub is_registered: bool,
    pub authorized_addresses: Vec<String>,
    pub created_at: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Payment {
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
    pub requires_proof: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Stats {
    pub total_users: u64,
    pub total_payments: u64,
    pub total_volume: Uint128,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const STATS: Item<Stats> = Item::new("stats");

pub const USERS: Map<&str, User> = Map::new("users");
pub const USERNAME_TO_ADDRESS: Map<&str, String> = Map::new("username_to_address");
pub const PAYMENTS: Map<&str, Payment> = Map::new("payments");
pub const USER_PAYMENTS: Map<&str, Vec<String>> = Map::new("user_payments");
pub const PENDING_BALANCES: Map<&str, Uint128> = Map::new("pending_balances");