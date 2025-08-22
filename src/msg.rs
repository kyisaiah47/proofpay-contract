use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::state::{User, FriendRequest, Payment, ProofType};
use cosmwasm_std::Coin;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    // User Management
    RegisterUser { 
        username: String, 
        display_name: String 
    },
    UpdateUserProfile { 
        display_name: Option<String>, 
        profile_picture: Option<String> 
    },
    
    // Friends System
    SendFriendRequest { 
        to_username: String 
    },
    AcceptFriendRequest { 
        from_username: String 
    },
    DeclineFriendRequest { 
        from_username: String 
    },
    RemoveFriend { 
        username: String 
    },
    
    // Payment System
    SendDirectPayment { 
        to_username: String, 
        amount: Coin,
        description: String, 
        proof_type: ProofType 
    },
    CreatePaymentRequest { 
        to_username: String, 
        amount: Coin,
        description: String, 
        proof_type: ProofType 
    },
    CreateHelpRequest { 
        to_username: String, 
        amount: Coin,
        description: String, 
        proof_type: ProofType 
    },
    SubmitProof { 
        payment_id: u64, 
        proof_data: String 
    },
    ApprovePayment { 
        payment_id: u64 
    },
    RejectPayment { 
        payment_id: u64 
    },
    CancelPayment { 
        payment_id: u64 
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // User Management
    GetUserByUsername { 
        username: String 
    },
    GetUserByWallet { 
        wallet_address: String 
    },
    IsUsernameAvailable { 
        username: String 
    },
    SearchUsers { 
        query: String 
    },
    
    // New username-specific queries
    GetUsernameByWallet { 
        wallet_address: String 
    },
    GetWalletByUsername { 
        username: String 
    },
    HasUsername { 
        wallet_address: String 
    },
    
    // Friends System
    GetUserFriends { 
        username: String 
    },
    GetPendingRequests { 
        username: String 
    },
    AreFriends { 
        username1: String, 
        username2: String 
    },
    
    // Payment System
    GetPaymentById { 
        payment_id: u64 
    },
    GetPaymentHistory { 
        username: String 
    },
    GetPendingPayments { 
        username: String 
    },
}

// Response Types
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserResponse {
    pub user: User,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UsersResponse {
    pub users: Vec<User>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UsernameAvailableResponse {
    pub available: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UsernameResponse {
    pub username: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WalletResponse {
    pub wallet_address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HasUsernameResponse {
    pub has_username: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FriendsResponse {
    pub friends: Vec<String>, // usernames
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FriendRequestsResponse {
    pub requests: Vec<FriendRequest>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AreFriendsResponse {
    pub are_friends: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PaymentResponse {
    pub payment: Payment,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PaymentsResponse {
    pub payments: Vec<Payment>,
}
