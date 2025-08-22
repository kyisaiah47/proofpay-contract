#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Order, Addr,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::*;
use crate::state::*;

const CONTRACT_NAME: &str = "crates.io:social-payment-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        owner: info.sender.clone(),
        next_payment_id: 1,
    };
    
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        // User Management
        ExecuteMsg::RegisterUser { username, display_name } => {
            execute_register_user(deps, env, info, username, display_name)
        }
        ExecuteMsg::UpdateUserProfile { display_name, profile_picture } => {
            execute_update_user_profile(deps, env, info, display_name, profile_picture)
        }
        
        // Friends System
        ExecuteMsg::SendFriendRequest { to_username } => {
            execute_send_friend_request(deps, env, info, to_username)
        }
        ExecuteMsg::AcceptFriendRequest { from_username } => {
            execute_accept_friend_request(deps, env, info, from_username)
        }
        ExecuteMsg::DeclineFriendRequest { from_username } => {
            execute_decline_friend_request(deps, env, info, from_username)
        }
        ExecuteMsg::RemoveFriend { username } => {
            execute_remove_friend(deps, env, info, username)
        }
        
        // Payment System
        ExecuteMsg::SendDirectPayment { to_username, amount, description, proof_type } => {
            execute_send_direct_payment(deps, env, info, to_username, amount, description, proof_type)
        }
        ExecuteMsg::CreatePaymentRequest { to_username, amount, description, proof_type } => {
            execute_create_payment_request(deps, env, info, to_username, amount, description, proof_type)
        }
        ExecuteMsg::CreateHelpRequest { to_username, amount, description, proof_type } => {
            execute_create_help_request(deps, env, info, to_username, amount, description, proof_type)
        }
        ExecuteMsg::SubmitProof { payment_id, proof_data } => {
            execute_submit_proof(deps, env, info, payment_id, proof_data)
        }
        ExecuteMsg::ApprovePayment { payment_id } => {
            execute_approve_payment(deps, env, info, payment_id)
        }
        ExecuteMsg::RejectPayment { payment_id } => {
            execute_reject_payment(deps, env, info, payment_id)
        }
        ExecuteMsg::CancelPayment { payment_id } => {
            execute_cancel_payment(deps, env, info, payment_id)
        }
    }
}

// Helper function to validate username format
fn validate_username(username: &str) -> Result<(), ContractError> {
    if username.len() < 3 || username.len() > 20 {
        return Err(ContractError::InvalidUsername {});
    }
    
    if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(ContractError::InvalidUsername {});
    }
    
    Ok(())
}

// Helper function to get username from wallet address
fn get_username_from_wallet(deps: &DepsMut, wallet: &Addr) -> Result<String, ContractError> {
    USERS_BY_WALLET.load(deps.storage, wallet.clone())
        .map_err(|_| ContractError::UserNotRegistered {})
}

// USER MANAGEMENT FUNCTIONS

pub fn execute_register_user(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    username: String,
    display_name: String,
) -> Result<Response, ContractError> {
    // Validate username format
    validate_username(&username)?;
    
    // Check if username is already taken
    if USERS_BY_USERNAME.may_load(deps.storage, username.clone())?.is_some() {
        return Err(ContractError::UsernameAlreadyTaken {});
    }
    
    // Check if wallet is already registered
    if USERS_BY_WALLET.may_load(deps.storage, info.sender.clone())?.is_some() {
        return Err(ContractError::WalletAlreadyRegistered {});
    }
    
    let user = User {
        wallet_address: info.sender.clone(),
        username: username.clone(),
        display_name,
        profile_picture: None,
        created_at: env.block.time.seconds(),
        updated_at: env.block.time.seconds(),
    };
    
    // Save user data
    USERS_BY_USERNAME.save(deps.storage, username.clone(), &user)?;
    USERS_BY_WALLET.save(deps.storage, info.sender.clone(), &username)?;
    
    Ok(Response::new()
        .add_attribute("action", "register_user")
        .add_attribute("username", username)
        .add_attribute("wallet", info.sender.as_str()))
}

pub fn execute_update_user_profile(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    display_name: Option<String>,
    profile_picture: Option<String>,
) -> Result<Response, ContractError> {
    let username = get_username_from_wallet(&deps, &info.sender)?;
    
    USERS_BY_USERNAME.update(deps.storage, username.clone(), |user| -> Result<_, ContractError> {
        let mut user = user.ok_or(ContractError::UserNotFound {})?;
        
        if let Some(new_display_name) = display_name {
            user.display_name = new_display_name;
        }
        
        if let Some(new_profile_picture) = profile_picture {
            user.profile_picture = Some(new_profile_picture);
        }
        
        user.updated_at = env.block.time.seconds();
        
        Ok(user)
    })?;
    
    Ok(Response::new()
        .add_attribute("action", "update_user_profile")
        .add_attribute("username", username))
}

// FRIENDS SYSTEM FUNCTIONS

pub fn execute_send_friend_request(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    to_username: String,
) -> Result<Response, ContractError> {
    let from_username = get_username_from_wallet(&deps, &info.sender)?;
    
    // Check if trying to add self
    if from_username == to_username {
        return Err(ContractError::CannotAddSelf {});
    }
    
    // Check if target user exists
    if USERS_BY_USERNAME.may_load(deps.storage, to_username.clone())?.is_none() {
        return Err(ContractError::UserNotFound {});
    }
    
    // Check if already friends
    let friendship_key1 = (from_username.clone(), to_username.clone());
    let friendship_key2 = (to_username.clone(), from_username.clone());
    
    if FRIENDSHIPS.may_load(deps.storage, friendship_key1)?.is_some() ||
       FRIENDSHIPS.may_load(deps.storage, friendship_key2)?.is_some() {
        return Err(ContractError::AlreadyFriends {});
    }
    
    // Check if friend request already exists
    let request_key = (from_username.clone(), to_username.clone());
    if FRIEND_REQUESTS.may_load(deps.storage, request_key.clone())?.is_some() {
        return Err(ContractError::FriendRequestAlreadyExists {});
    }
    
    let friend_request = FriendRequest {
        from_username: from_username.clone(),
        to_username: to_username.clone(),
        status: FriendRequestStatus::Pending,
        created_at: env.block.time.seconds(),
        updated_at: env.block.time.seconds(),
    };
    
    FRIEND_REQUESTS.save(deps.storage, request_key, &friend_request)?;
    
    Ok(Response::new()
        .add_attribute("action", "send_friend_request")
        .add_attribute("from", from_username)
        .add_attribute("to", to_username))
}

pub fn execute_accept_friend_request(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    from_username: String,
) -> Result<Response, ContractError> {
    let to_username = get_username_from_wallet(&deps, &info.sender)?;
    
    let request_key = (from_username.clone(), to_username.clone());
    let _friend_request = FRIEND_REQUESTS.load(deps.storage, request_key.clone())
        .map_err(|_| ContractError::FriendRequestNotFound {})?;
    
    // Update friend request status
    FRIEND_REQUESTS.update(deps.storage, request_key.clone(), |req| -> Result<_, ContractError> {
        let mut req = req.ok_or(ContractError::FriendRequestNotFound {})?;
        req.status = FriendRequestStatus::Accepted;
        req.updated_at = env.block.time.seconds();
        Ok(req)
    })?;
    
    // Create friendship (store both directions for easier lookup)
    let friendship = Friendship {
        user1: from_username.clone(),
        user2: to_username.clone(),
        created_at: env.block.time.seconds(),
    };
    
    FRIENDSHIPS.save(deps.storage, (from_username.clone(), to_username.clone()), &friendship)?;
    FRIENDSHIPS.save(deps.storage, (to_username.clone(), from_username.clone()), &friendship)?;
    
    Ok(Response::new()
        .add_attribute("action", "accept_friend_request")
        .add_attribute("from", from_username)
        .add_attribute("to", to_username))
}

pub fn execute_decline_friend_request(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    from_username: String,
) -> Result<Response, ContractError> {
    let to_username = get_username_from_wallet(&deps, &info.sender)?;
    
    let request_key = (from_username.clone(), to_username.clone());
    
    FRIEND_REQUESTS.update(deps.storage, request_key, |req| -> Result<_, ContractError> {
        let mut req = req.ok_or(ContractError::FriendRequestNotFound {})?;
        req.status = FriendRequestStatus::Declined;
        req.updated_at = env.block.time.seconds();
        Ok(req)
    })?;
    
    Ok(Response::new()
        .add_attribute("action", "decline_friend_request")
        .add_attribute("from", from_username)
        .add_attribute("to", to_username))
}

pub fn execute_remove_friend(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    friend_username: String,
) -> Result<Response, ContractError> {
    let username = get_username_from_wallet(&deps, &info.sender)?;
    
    // Check if they are friends
    let friendship_key1 = (username.clone(), friend_username.clone());
    let friendship_key2 = (friend_username.clone(), username.clone());
    
    if FRIENDSHIPS.may_load(deps.storage, friendship_key1.clone())?.is_none() {
        return Err(ContractError::NotFriends {});
    }
    
    // Remove friendship (both directions)
    FRIENDSHIPS.remove(deps.storage, friendship_key1);
    FRIENDSHIPS.remove(deps.storage, friendship_key2);
    
    Ok(Response::new()
        .add_attribute("action", "remove_friend")
        .add_attribute("user", username)
        .add_attribute("removed_friend", friend_username))
}

// PAYMENT SYSTEM FUNCTIONS

pub fn execute_send_direct_payment(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    to_username: String,
    amount: cosmwasm_std::Coin,
    description: String,
    proof_type: ProofType,
) -> Result<Response, ContractError> {
    let from_username = get_username_from_wallet(&deps, &info.sender)?;
    
    // Validate payment
    if from_username == to_username {
        return Err(ContractError::CannotPaySelf {});
    }
    
    // Check if recipient exists
    let recipient = USERS_BY_USERNAME.load(deps.storage, to_username.clone())
        .map_err(|_| ContractError::UserNotFound {})?;
    
    // Validate payment amount
    if amount.amount.is_zero() {
        return Err(ContractError::InvalidPaymentAmount {});
    }
    
    // Check if sufficient funds were sent
    let sent_amount = info.funds.iter()
        .find(|coin| coin.denom == amount.denom)
        .map(|coin| coin.amount)
        .unwrap_or_default();
    
    if sent_amount < amount.amount {
        return Err(ContractError::InsufficientFunds {});
    }
    
    let mut state = STATE.load(deps.storage)?;
    let payment_id = state.next_payment_id;
    state.next_payment_id += 1;
    STATE.save(deps.storage, &state)?;
    
    let payment = Payment {
        id: payment_id,
        from_username: from_username.clone(),
        to_username: to_username.clone(),
        amount,
        description,
        payment_type: PaymentType::DirectPayment,
        proof_type: proof_type.clone(),
        proof_data: None,
        status: if matches!(proof_type, ProofType::None) { 
            PaymentStatus::Completed 
        } else { 
            PaymentStatus::Pending 
        },
        created_at: env.block.time.seconds(),
        updated_at: env.block.time.seconds(),
    };
    
    PAYMENTS.save(deps.storage, payment_id, &payment)?;
    USER_PAYMENTS.save(deps.storage, (from_username.clone(), payment_id), &true)?;
    USER_PAYMENTS.save(deps.storage, (to_username.clone(), payment_id), &true)?;
    
    let mut response = Response::new()
        .add_attribute("action", "send_direct_payment")
        .add_attribute("from", from_username)
        .add_attribute("to", to_username.clone())
        .add_attribute("payment_id", payment_id.to_string())
        .add_attribute("amount", payment.amount.to_string());
    
    // If no proof required, send payment immediately
    if matches!(proof_type, ProofType::None) {
        let payment_msg = CosmosMsg::Bank(BankMsg::Send {
            to_address: recipient.wallet_address.to_string(),
            amount: vec![payment.amount],
        });
        response = response.add_message(payment_msg);
    }
    
    Ok(response)
}

pub fn execute_create_payment_request(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    to_username: String,
    amount: cosmwasm_std::Coin,
    description: String,
    proof_type: ProofType,
) -> Result<Response, ContractError> {
    let from_username = get_username_from_wallet(&deps, &info.sender)?;
    
    // Validate
    if from_username == to_username {
        return Err(ContractError::CannotPaySelf {});
    }
    
    // Check if recipient exists
    if USERS_BY_USERNAME.may_load(deps.storage, to_username.clone())?.is_none() {
        return Err(ContractError::UserNotFound {});
    }
    
    let mut state = STATE.load(deps.storage)?;
    let payment_id = state.next_payment_id;
    state.next_payment_id += 1;
    STATE.save(deps.storage, &state)?;
    
    let payment = Payment {
        id: payment_id,
        from_username: from_username.clone(),
        to_username: to_username.clone(),
        amount,
        description,
        payment_type: PaymentType::PaymentRequest,
        proof_type,
        proof_data: None,
        status: PaymentStatus::Pending,
        created_at: env.block.time.seconds(),
        updated_at: env.block.time.seconds(),
    };
    
    PAYMENTS.save(deps.storage, payment_id, &payment)?;
    USER_PAYMENTS.save(deps.storage, (from_username.clone(), payment_id), &true)?;
    USER_PAYMENTS.save(deps.storage, (to_username.clone(), payment_id), &true)?;
    
    Ok(Response::new()
        .add_attribute("action", "create_payment_request")
        .add_attribute("from", from_username)
        .add_attribute("to", to_username)
        .add_attribute("payment_id", payment_id.to_string())
        .add_attribute("amount", payment.amount.to_string()))
}

pub fn execute_create_help_request(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    to_username: String,
    amount: cosmwasm_std::Coin,
    description: String,
    proof_type: ProofType,
) -> Result<Response, ContractError> {
    let from_username = get_username_from_wallet(&deps, &info.sender)?;
    
    // Validate
    if from_username == to_username {
        return Err(ContractError::CannotPaySelf {});
    }
    
    // Check if recipient exists
    if USERS_BY_USERNAME.may_load(deps.storage, to_username.clone())?.is_none() {
        return Err(ContractError::UserNotFound {});
    }
    
    // Check if sufficient funds were sent for escrow
    let sent_amount = info.funds.iter()
        .find(|coin| coin.denom == amount.denom)
        .map(|coin| coin.amount)
        .unwrap_or_default();
    
    if sent_amount < amount.amount {
        return Err(ContractError::InsufficientFunds {});
    }
    
    let mut state = STATE.load(deps.storage)?;
    let payment_id = state.next_payment_id;
    state.next_payment_id += 1;
    STATE.save(deps.storage, &state)?;
    
    let payment = Payment {
        id: payment_id,
        from_username: from_username.clone(),
        to_username: to_username.clone(),
        amount,
        description,
        payment_type: PaymentType::HelpRequest,
        proof_type,
        proof_data: None,
        status: PaymentStatus::Pending,
        created_at: env.block.time.seconds(),
        updated_at: env.block.time.seconds(),
    };
    
    PAYMENTS.save(deps.storage, payment_id, &payment)?;
    USER_PAYMENTS.save(deps.storage, (from_username.clone(), payment_id), &true)?;
    USER_PAYMENTS.save(deps.storage, (to_username.clone(), payment_id), &true)?;
    
    Ok(Response::new()
        .add_attribute("action", "create_help_request")
        .add_attribute("from", from_username)
        .add_attribute("to", to_username)
        .add_attribute("payment_id", payment_id.to_string())
        .add_attribute("amount", payment.amount.to_string()))
}

pub fn execute_submit_proof(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    payment_id: u64,
    proof_data: String,
) -> Result<Response, ContractError> {
    let username = get_username_from_wallet(&deps, &info.sender)?;
    
    PAYMENTS.update(deps.storage, payment_id, |payment| -> Result<_, ContractError> {
        let mut payment = payment.ok_or(ContractError::PaymentNotFound {})?;
        
        // Check authorization - only the recipient can submit proof
        if payment.to_username != username {
            return Err(ContractError::PaymentNotAuthorized {});
        }
        
        // Check if proof is required
        if matches!(payment.proof_type, ProofType::None) {
            return Err(ContractError::NoProofRequired {});
        }
        
        // Check payment status
        if !matches!(payment.status, PaymentStatus::Pending) {
            return Err(ContractError::PaymentAlreadyCompleted {});
        }
        
        payment.proof_data = Some(proof_data);
        payment.status = PaymentStatus::ProofSubmitted;
        payment.updated_at = env.block.time.seconds();
        
        Ok(payment)
    })?;
    
    Ok(Response::new()
        .add_attribute("action", "submit_proof")
        .add_attribute("payment_id", payment_id.to_string())
        .add_attribute("submitter", username))
}

pub fn execute_approve_payment(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    payment_id: u64,
) -> Result<Response, ContractError> {
    let username = get_username_from_wallet(&deps, &info.sender)?;
    
    let payment = PAYMENTS.load(deps.storage, payment_id)
        .map_err(|_| ContractError::PaymentNotFound {})?;
    
    // Check authorization based on payment type
    let authorized = match payment.payment_type {
        PaymentType::DirectPayment => payment.from_username == username,
        PaymentType::PaymentRequest => payment.to_username == username,
        PaymentType::HelpRequest => payment.from_username == username,
    };
    
    if !authorized {
        return Err(ContractError::PaymentNotAuthorized {});
    }
    
    // Check if proof is required and submitted
    if !matches!(payment.proof_type, ProofType::None) && 
       !matches!(payment.status, PaymentStatus::ProofSubmitted) {
        return Err(ContractError::ProofRequired {});
    }
    
    // Update payment status
    PAYMENTS.update(deps.storage, payment_id, |payment| -> Result<_, ContractError> {
        let mut payment = payment.ok_or(ContractError::PaymentNotFound {})?;
        
        if matches!(payment.status, PaymentStatus::Completed) {
            return Err(ContractError::PaymentAlreadyCompleted {});
        }
        
        payment.status = PaymentStatus::Completed;
        payment.updated_at = env.block.time.seconds();
        
        Ok(payment)
    })?;
    
    let recipient = USERS_BY_USERNAME.load(deps.storage, payment.to_username.clone())?;
    
    // Send payment to recipient
    let payment_msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: recipient.wallet_address.to_string(),
        amount: vec![payment.amount],
    });
    
    Ok(Response::new()
        .add_message(payment_msg)
        .add_attribute("action", "approve_payment")
        .add_attribute("payment_id", payment_id.to_string())
        .add_attribute("approver", username))
}

pub fn execute_reject_payment(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    payment_id: u64,
) -> Result<Response, ContractError> {
    let username = get_username_from_wallet(&deps, &info.sender)?;
    
    let payment = PAYMENTS.load(deps.storage, payment_id)
        .map_err(|_| ContractError::PaymentNotFound {})?;
    
    // Check authorization based on payment type
    let authorized = match payment.payment_type {
        PaymentType::DirectPayment => payment.from_username == username,
        PaymentType::PaymentRequest => payment.to_username == username,
        PaymentType::HelpRequest => payment.from_username == username,
    };
    
    if !authorized {
        return Err(ContractError::PaymentNotAuthorized {});
    }
    
    // Update payment status
    PAYMENTS.update(deps.storage, payment_id, |payment| -> Result<_, ContractError> {
        let mut payment = payment.ok_or(ContractError::PaymentNotFound {})?;
        
        if matches!(payment.status, PaymentStatus::Completed | PaymentStatus::Cancelled) {
            return Err(ContractError::PaymentAlreadyCompleted {});
        }
        
        payment.status = PaymentStatus::Rejected;
        payment.updated_at = env.block.time.seconds();
        
        Ok(payment)
    })?;
    
    Ok(Response::new()
        .add_attribute("action", "reject_payment")
        .add_attribute("payment_id", payment_id.to_string())
        .add_attribute("rejector", username))
}

pub fn execute_cancel_payment(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    payment_id: u64,
) -> Result<Response, ContractError> {
    let username = get_username_from_wallet(&deps, &info.sender)?;
    
    let payment = PAYMENTS.load(deps.storage, payment_id)
        .map_err(|_| ContractError::PaymentNotFound {})?;
    
    // Only sender can cancel
    if payment.from_username != username {
        return Err(ContractError::OnlySenderCanCancel {});
    }
    
    // Update payment status
    PAYMENTS.update(deps.storage, payment_id, |payment| -> Result<_, ContractError> {
        let mut payment = payment.ok_or(ContractError::PaymentNotFound {})?;
        
        if matches!(payment.status, PaymentStatus::Completed) {
            return Err(ContractError::PaymentAlreadyCompleted {});
        }
        
        if matches!(payment.status, PaymentStatus::Cancelled) {
            return Err(ContractError::PaymentAlreadyCancelled {});
        }
        
        payment.status = PaymentStatus::Cancelled;
        payment.updated_at = env.block.time.seconds();
        
        Ok(payment)
    })?;
    
    let sender = USERS_BY_USERNAME.load(deps.storage, payment.from_username.clone())?;
    
    // Refund to sender (for HelpRequest type)
    let mut response = Response::new()
        .add_attribute("action", "cancel_payment")
        .add_attribute("payment_id", payment_id.to_string())
        .add_attribute("canceller", username);
    
    if matches!(payment.payment_type, PaymentType::HelpRequest) {
        let refund_msg = CosmosMsg::Bank(BankMsg::Send {
            to_address: sender.wallet_address.to_string(),
            amount: vec![payment.amount],
        });
        response = response.add_message(refund_msg);
    }
    
    Ok(response)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        // User Management
        QueryMsg::GetUserByUsername { username } => query_user_by_username(deps, username),
        QueryMsg::GetUserByWallet { wallet_address } => query_user_by_wallet(deps, wallet_address),
        QueryMsg::IsUsernameAvailable { username } => query_username_available(deps, username),
        QueryMsg::SearchUsers { query } => query_search_users(deps, query),
        
        // Friends System
        QueryMsg::GetUserFriends { username } => query_user_friends(deps, username),
        QueryMsg::GetPendingRequests { username } => query_pending_requests(deps, username),
        QueryMsg::AreFriends { username1, username2 } => query_are_friends(deps, username1, username2),
        
        // Payment System
        QueryMsg::GetPaymentById { payment_id } => query_payment_by_id(deps, payment_id),
        QueryMsg::GetPaymentHistory { username } => query_payment_history(deps, username),
        QueryMsg::GetPendingPayments { username } => query_pending_payments(deps, username),
    }
}

// USER MANAGEMENT QUERIES

fn query_user_by_username(deps: Deps, username: String) -> StdResult<Binary> {
    let user = USERS_BY_USERNAME.load(deps.storage, username)?;
    to_json_binary(&UserResponse { user })
}

fn query_user_by_wallet(deps: Deps, wallet_address: String) -> StdResult<Binary> {
    let wallet_addr = deps.api.addr_validate(&wallet_address)?;
    let username = USERS_BY_WALLET.load(deps.storage, wallet_addr)?;
    let user = USERS_BY_USERNAME.load(deps.storage, username)?;
    to_json_binary(&UserResponse { user })
}

fn query_username_available(deps: Deps, username: String) -> StdResult<Binary> {
    let available = USERS_BY_USERNAME.may_load(deps.storage, username)?.is_none();
    to_json_binary(&UsernameAvailableResponse { available })
}

fn query_search_users(deps: Deps, query: String) -> StdResult<Binary> {
    let query_lower = query.to_lowercase();
    let users: StdResult<Vec<User>> = USERS_BY_USERNAME
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| item.map(|(_, user)| user))
        .filter(|user| {
            user.as_ref()
                .map(|u| {
                    u.username.to_lowercase().contains(&query_lower) ||
                    u.display_name.to_lowercase().contains(&query_lower)
                })
                .unwrap_or(false)
        })
        .collect();
    to_json_binary(&UsersResponse { users: users? })
}

// FRIENDS SYSTEM QUERIES

fn query_user_friends(deps: Deps, username: String) -> StdResult<Binary> {
    let friends: StdResult<Vec<String>> = FRIENDSHIPS
        .prefix(username)
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| item.map(|(friend_username, _)| friend_username))
        .collect();
    to_json_binary(&FriendsResponse { friends: friends? })
}

fn query_pending_requests(deps: Deps, username: String) -> StdResult<Binary> {
    let mut requests = Vec::new();
    
    // Get requests sent TO this user
    for item in FRIEND_REQUESTS.range(deps.storage, None, None, Order::Ascending) {
        let ((_from, to), request) = item?;
        if to == username && matches!(request.status, FriendRequestStatus::Pending) {
            requests.push(request);
        }
    }
    
    to_json_binary(&FriendRequestsResponse { requests })
}

fn query_are_friends(deps: Deps, username1: String, username2: String) -> StdResult<Binary> {
    let are_friends = FRIENDSHIPS
        .may_load(deps.storage, (username1, username2))?
        .is_some();
    to_json_binary(&AreFriendsResponse { are_friends })
}

// PAYMENT SYSTEM QUERIES

fn query_payment_by_id(deps: Deps, payment_id: u64) -> StdResult<Binary> {
    let payment = PAYMENTS.load(deps.storage, payment_id)?;
    to_json_binary(&PaymentResponse { payment })
}

fn query_payment_history(deps: Deps, username: String) -> StdResult<Binary> {
    let mut payments = Vec::new();
    
    // Get all payments for this user
    for item in USER_PAYMENTS.prefix(username).range(deps.storage, None, None, Order::Ascending) {
        let (payment_id, _) = item?;
        if let Ok(payment) = PAYMENTS.load(deps.storage, payment_id) {
            payments.push(payment);
        }
    }
    
    to_json_binary(&PaymentsResponse { payments })
}

fn query_pending_payments(deps: Deps, username: String) -> StdResult<Binary> {
    let mut payments = Vec::new();
    
    // Get all payments for this user that are pending
    for item in USER_PAYMENTS.prefix(username).range(deps.storage, None, None, Order::Ascending) {
        let (payment_id, _) = item?;
        if let Ok(payment) = PAYMENTS.load(deps.storage, payment_id) {
            if matches!(payment.status, PaymentStatus::Pending | PaymentStatus::ProofSubmitted) {
                payments.push(payment);
            }
        }
    }
    
    to_json_binary(&PaymentsResponse { payments })
}
