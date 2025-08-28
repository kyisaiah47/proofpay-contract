use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, Uint128, coins, BankMsg,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, QueryMsg, UserResponse, PaymentResponse,
    StatsResponse, PaymentStatus, ProofType,
};
use crate::state::{
    Config, User, Payment, Stats, CONFIG, STATS, USERS, USERNAME_TO_ADDRESS,
    PAYMENTS, USER_PAYMENTS, PENDING_BALANCES,
};

const CONTRACT_NAME: &str = "proofpay-cosmwasm";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const MAX_DESCRIPTION_LENGTH: usize = 500;
const MAX_PROOF_SIZE: usize = 10000;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        admin: msg.admin.unwrap_or_else(|| info.sender.to_string()),
    };
    
    let stats = Stats {
        total_users: 0,
        total_payments: 0,
        total_volume: Uint128::zero(),
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &config)?;
    STATS.save(deps.storage, &stats)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", config.admin))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::RegisterUser { username } => execute_register_user(deps, env, info, username),
        ExecuteMsg::AddAuthorizedAddress { address } => {
            execute_add_authorized_address(deps, info, address)
        }
        ExecuteMsg::RemoveAuthorizedAddress { address } => {
            execute_remove_authorized_address(deps, info, address)
        }
        ExecuteMsg::CreatePayment {
            recipient,
            amount,
            token,
            proof_type,
            description,
            requires_proof,
        } => execute_create_payment(
            deps,
            env,
            info,
            recipient,
            amount,
            token,
            proof_type,
            description,
            requires_proof,
        ),
        ExecuteMsg::SubmitProof { payment_id, proof } => {
            execute_submit_proof(deps, info, payment_id, proof)
        }
        ExecuteMsg::CompletePayment { payment_id } => {
            execute_complete_payment(deps, env, info, payment_id)
        }
        ExecuteMsg::DisputePayment { payment_id, reason } => {
            execute_dispute_payment(deps, env, info, payment_id, reason)
        }
        ExecuteMsg::CancelPayment { payment_id } => {
            execute_cancel_payment(deps, env, info, payment_id)
        }
        ExecuteMsg::SendIbcPayment { .. } => {
            // IBC implementation will be added later
            Err(ContractError::CrossChainError {
                msg: "IBC not implemented yet".to_string(),
            })
        }
    }
}

fn execute_register_user(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    username: String,
) -> Result<Response, ContractError> {
    let sender = info.sender.to_string();
    
    // Check if user already registered
    if USERS.has(deps.storage, &sender) {
        return Err(ContractError::UserAlreadyRegistered {});
    }

    // Validate username
    if username.is_empty() || username.len() > 32 {
        return Err(ContractError::InvalidUsername {});
    }

    // Check if username is taken
    if USERNAME_TO_ADDRESS.has(deps.storage, &username) {
        return Err(ContractError::UsernameTaken {});
    }

    let user = User {
        username: username.clone(),
        is_registered: true,
        authorized_addresses: vec![],
        created_at: env.block.time.seconds(),
    };

    USERS.save(deps.storage, &sender, &user)?;
    USERNAME_TO_ADDRESS.save(deps.storage, &username, &sender)?;

    // Update stats
    let mut stats = STATS.load(deps.storage)?;
    stats.total_users += 1;
    STATS.save(deps.storage, &stats)?;

    Ok(Response::new()
        .add_attribute("method", "register_user")
        .add_attribute("user", sender)
        .add_attribute("username", username))
}

fn execute_add_authorized_address(
    deps: DepsMut,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    let sender = info.sender.to_string();
    
    // Check if user is registered
    let mut user = USERS
        .may_load(deps.storage, &sender)?
        .ok_or(ContractError::UserNotRegistered {})?;

    // Validate address
    deps.api.addr_validate(&address)?;
    
    if address == sender {
        return Err(ContractError::CannotPaySelf {});
    }

    if user.authorized_addresses.contains(&address) {
        return Err(ContractError::AddressAlreadyAuthorized {});
    }

    user.authorized_addresses.push(address.clone());
    USERS.save(deps.storage, &sender, &user)?;

    Ok(Response::new()
        .add_attribute("method", "add_authorized_address")
        .add_attribute("user", sender)
        .add_attribute("authorized_address", address))
}

fn execute_remove_authorized_address(
    deps: DepsMut,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    let sender = info.sender.to_string();
    
    let mut user = USERS
        .may_load(deps.storage, &sender)?
        .ok_or(ContractError::UserNotRegistered {})?;

    if !user.authorized_addresses.contains(&address) {
        return Err(ContractError::AddressNotAuthorized {});
    }

    user.authorized_addresses.retain(|addr| addr != &address);
    USERS.save(deps.storage, &sender, &user)?;

    Ok(Response::new()
        .add_attribute("method", "remove_authorized_address")
        .add_attribute("user", sender)
        .add_attribute("removed_address", address))
}

fn execute_create_payment(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
    token: Option<String>,
    proof_type: Option<ProofType>,
    description: Option<String>,
    requires_proof: bool,
) -> Result<Response, ContractError> {
    let sender = info.sender.to_string();

    // Validate recipient
    deps.api.addr_validate(&recipient)?;
    
    if recipient == sender {
        return Err(ContractError::CannotPaySelf {});
    }

    if amount.is_zero() {
        return Err(ContractError::InvalidAmount {});
    }

    // Validate description length
    if let Some(ref desc) = description {
        if desc.len() > MAX_DESCRIPTION_LENGTH {
            return Err(ContractError::InvalidDescriptionLength {});
        }
    }

    // Generate payment ID
    let payment_id = format!(
        "{}:{}:{}:{}",
        sender,
        recipient,
        amount,
        env.block.time.seconds()
    );

    let payment = Payment {
        id: payment_id.clone(),
        sender: sender.clone(),
        recipient: recipient.clone(),
        amount,
        token,
        status: PaymentStatus::Pending,
        proof_type,
        proof_data: None,
        description,
        created_at: env.block.time.seconds(),
        completed_at: None,
        requires_proof,
    };

    PAYMENTS.save(deps.storage, &payment_id, &payment)?;

    // Update user payments
    let mut sender_payments = USER_PAYMENTS
        .may_load(deps.storage, &sender)?
        .unwrap_or_default();
    sender_payments.push(payment_id.clone());
    USER_PAYMENTS.save(deps.storage, &sender, &sender_payments)?;

    let mut recipient_payments = USER_PAYMENTS
        .may_load(deps.storage, &recipient)?
        .unwrap_or_default();
    recipient_payments.push(payment_id.clone());
    USER_PAYMENTS.save(deps.storage, &recipient, &recipient_payments)?;

    // Update pending balances
    let current_pending = PENDING_BALANCES
        .may_load(deps.storage, &recipient)?
        .unwrap_or_default();
    PENDING_BALANCES.save(deps.storage, &recipient, &(current_pending + amount))?;

    // Update stats
    let mut stats = STATS.load(deps.storage)?;
    stats.total_payments += 1;
    stats.total_volume += amount;
    STATS.save(deps.storage, &stats)?;

    Ok(Response::new()
        .add_attribute("method", "create_payment")
        .add_attribute("payment_id", payment_id)
        .add_attribute("sender", sender)
        .add_attribute("recipient", recipient)
        .add_attribute("amount", amount.to_string()))
}

fn execute_submit_proof(
    deps: DepsMut,
    info: MessageInfo,
    payment_id: String,
    proof: Binary,
) -> Result<Response, ContractError> {
    let sender = info.sender.to_string();
    
    let mut payment = PAYMENTS
        .may_load(deps.storage, &payment_id)?
        .ok_or(ContractError::PaymentNotFound {})?;

    if payment.status != PaymentStatus::Pending {
        return Err(ContractError::PaymentNotPending {});
    }

    if sender != payment.recipient {
        return Err(ContractError::Unauthorized {});
    }

    if !payment.requires_proof {
        return Err(ContractError::InvalidProof {});
    }

    if proof.len() > MAX_PROOF_SIZE {
        return Err(ContractError::InvalidProof {});
    }

    payment.proof_data = Some(proof);
    PAYMENTS.save(deps.storage, &payment_id, &payment)?;

    Ok(Response::new()
        .add_attribute("method", "submit_proof")
        .add_attribute("payment_id", payment_id)
        .add_attribute("recipient", sender))
}

fn execute_complete_payment(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    payment_id: String,
) -> Result<Response, ContractError> {
    let sender = info.sender.to_string();
    
    let mut payment = PAYMENTS
        .may_load(deps.storage, &payment_id)?
        .ok_or(ContractError::PaymentNotFound {})?;

    if payment.status != PaymentStatus::Pending {
        return Err(ContractError::PaymentNotPending {});
    }

    // Check authorization
    let is_sender = sender == payment.sender;
    let is_recipient = sender == payment.recipient;
    
    if !is_sender && !is_recipient {
        return Err(ContractError::Unauthorized {});
    }

    if payment.requires_proof {
        if payment.proof_data.is_none() {
            return Err(ContractError::ProofRequired {});
        }
        // Only sender can complete after proof is submitted
        if !is_sender {
            return Err(ContractError::Unauthorized {});
        }
    }

    payment.status = PaymentStatus::Completed;
    payment.completed_at = Some(env.block.time.seconds());
    PAYMENTS.save(deps.storage, &payment_id, &payment)?;

    // Update pending balances
    let current_pending = PENDING_BALANCES
        .may_load(deps.storage, &payment.recipient)?
        .unwrap_or_default();
    PENDING_BALANCES.save(
        deps.storage,
        &payment.recipient,
        &(current_pending - payment.amount),
    )?;

    let mut response = Response::new()
        .add_attribute("method", "complete_payment")
        .add_attribute("payment_id", payment_id)
        .add_attribute("recipient", payment.recipient.clone())
        .add_attribute("amount", payment.amount.to_string());

    // Transfer funds (native token only for now)
    if payment.token.is_none() {
        let transfer_msg = BankMsg::Send {
            to_address: payment.recipient,
            amount: coins(payment.amount.u128(), "uosmo"), // Default to uosmo
        };
        response = response.add_message(transfer_msg);
    }

    Ok(response)
}

fn execute_dispute_payment(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    payment_id: String,
    reason: String,
) -> Result<Response, ContractError> {
    let sender = info.sender.to_string();
    
    let mut payment = PAYMENTS
        .may_load(deps.storage, &payment_id)?
        .ok_or(ContractError::PaymentNotFound {})?;

    if payment.status != PaymentStatus::Pending {
        return Err(ContractError::PaymentNotPending {});
    }

    // Check if sender is participant
    if sender != payment.sender && sender != payment.recipient {
        return Err(ContractError::Unauthorized {});
    }

    if reason.is_empty() || reason.len() > 200 {
        return Err(ContractError::InvalidDisputeReason {});
    }

    payment.status = PaymentStatus::Disputed;
    PAYMENTS.save(deps.storage, &payment_id, &payment)?;

    Ok(Response::new()
        .add_attribute("method", "dispute_payment")
        .add_attribute("payment_id", payment_id)
        .add_attribute("disputant", sender)
        .add_attribute("reason", reason)
        .add_attribute("timestamp", env.block.time.seconds().to_string()))
}

fn execute_cancel_payment(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    payment_id: String,
) -> Result<Response, ContractError> {
    let sender = info.sender.to_string();
    
    let mut payment = PAYMENTS
        .may_load(deps.storage, &payment_id)?
        .ok_or(ContractError::PaymentNotFound {})?;

    if payment.status != PaymentStatus::Pending {
        return Err(ContractError::PaymentNotPending {});
    }

    if sender != payment.sender {
        return Err(ContractError::Unauthorized {});
    }

    payment.status = PaymentStatus::Cancelled;
    PAYMENTS.save(deps.storage, &payment_id, &payment)?;

    // Update pending balances
    let current_pending = PENDING_BALANCES
        .may_load(deps.storage, &payment.recipient)?
        .unwrap_or_default();
    PENDING_BALANCES.save(
        deps.storage,
        &payment.recipient,
        &(current_pending - payment.amount),
    )?;

    let mut response = Response::new()
        .add_attribute("method", "cancel_payment")
        .add_attribute("payment_id", payment_id)
        .add_attribute("sender", payment.sender.clone())
        .add_attribute("amount", payment.amount.to_string())
        .add_attribute("timestamp", env.block.time.seconds().to_string());

    // Refund funds (native token only for now)
    if payment.token.is_none() {
        let refund_msg = BankMsg::Send {
            to_address: payment.sender,
            amount: coins(payment.amount.u128(), "uosmo"), // Default to uosmo
        };
        response = response.add_message(refund_msg);
    }

    Ok(response)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetUser { address } => to_binary(&query_user(deps, address)?),
        QueryMsg::GetUserByUsername { username } => {
            to_binary(&query_user_by_username(deps, username)?)
        }
        QueryMsg::IsUsernameAvailable { username } => {
            to_binary(&query_is_username_available(deps, username)?)
        }
        QueryMsg::IsAuthorized { user, address } => {
            to_binary(&query_is_authorized(deps, user, address)?)
        }
        QueryMsg::GetPayment { payment_id } => to_binary(&query_payment(deps, payment_id)?),
        QueryMsg::GetUserPayments { user } => to_binary(&query_user_payments(deps, user)?),
        QueryMsg::GetStats {} => to_binary(&query_stats(deps)?),
    }
}

fn query_user(deps: Deps, address: String) -> StdResult<UserResponse> {
    let user = USERS.load(deps.storage, &address)?;
    Ok(UserResponse {
        username: user.username,
        is_registered: user.is_registered,
        created_at: user.created_at,
    })
}

fn query_user_by_username(deps: Deps, username: String) -> StdResult<String> {
    let address = USERNAME_TO_ADDRESS.load(deps.storage, &username)?;
    Ok(address)
}

fn query_is_username_available(deps: Deps, username: String) -> StdResult<bool> {
    Ok(!USERNAME_TO_ADDRESS.has(deps.storage, &username))
}

fn query_is_authorized(deps: Deps, user: String, address: String) -> StdResult<bool> {
    if user == address {
        return Ok(true);
    }
    
    if let Ok(user_data) = USERS.load(deps.storage, &user) {
        Ok(user_data.authorized_addresses.contains(&address))
    } else {
        Ok(false)
    }
}

fn query_payment(deps: Deps, payment_id: String) -> StdResult<PaymentResponse> {
    let payment = PAYMENTS.load(deps.storage, &payment_id)?;
    Ok(PaymentResponse {
        id: payment.id,
        sender: payment.sender,
        recipient: payment.recipient,
        amount: payment.amount,
        token: payment.token,
        status: payment.status,
        proof_type: payment.proof_type,
        proof_data: payment.proof_data,
        description: payment.description,
        created_at: payment.created_at,
        completed_at: payment.completed_at,
    })
}

fn query_user_payments(deps: Deps, user: String) -> StdResult<Vec<String>> {
    let payments = USER_PAYMENTS
        .may_load(deps.storage, &user)?
        .unwrap_or_default();
    Ok(payments)
}

fn query_stats(deps: Deps) -> StdResult<StatsResponse> {
    let stats = STATS.load(deps.storage)?;
    Ok(StatsResponse {
        total_users: stats.total_users,
        total_payments: stats.total_payments,
        total_volume: stats.total_volume,
    })
}