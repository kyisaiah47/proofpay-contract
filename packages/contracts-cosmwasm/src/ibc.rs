use cosmwasm_std::{
    entry_point, to_binary, Binary, DepsMut, Env, IbcBasicResponse, IbcChannelCloseMsg,
    IbcChannelConnectMsg, IbcChannelOpenMsg, IbcChannelOpenResponse, IbcOrder, IbcPacketAckMsg,
    IbcPacketReceiveMsg, IbcPacketTimeoutMsg, IbcReceiveResponse, Never, StdResult,
};

use crate::error::ContractError;
use crate::msg::CrossChainPaymentPacket;

pub const IBC_VERSION: &str = "proofpay-1";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_open(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelOpenMsg,
) -> Result<IbcChannelOpenResponse, ContractError> {
    validate_order_and_version(msg.channel(), msg.counterparty_version())?;
    Ok(IbcChannelOpenResponse::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_connect(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> Result<IbcBasicResponse, ContractError> {
    validate_order_and_version(msg.channel(), msg.counterparty_version())?;
    
    Ok(IbcBasicResponse::new()
        .add_attribute("action", "ibc_channel_connect")
        .add_attribute("channel", msg.channel().endpoint.channel_id.as_str()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_close(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelCloseMsg,
) -> Result<IbcBasicResponse, ContractError> {
    Ok(IbcBasicResponse::new()
        .add_attribute("action", "ibc_channel_close")
        .add_attribute("channel", msg.channel().endpoint.channel_id.as_str()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_receive(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, Never> {
    let packet: CrossChainPaymentPacket = match from_binary(&msg.packet.data) {
        Ok(packet) => packet,
        Err(err) => {
            return Ok(IbcReceiveResponse::new()
                .set_ack(ack_fail(format!("Invalid packet data: {}", err)))
                .add_attribute("action", "ibc_packet_receive")
                .add_attribute("error", "invalid_packet_data"));
        }
    };

    match process_received_payment(deps, env, packet) {
        Ok(payment_id) => Ok(IbcReceiveResponse::new()
            .set_ack(ack_success())
            .add_attribute("action", "ibc_packet_receive")
            .add_attribute("payment_id", payment_id)
            .add_attribute("success", "true")),
        Err(err) => Ok(IbcReceiveResponse::new()
            .set_ack(ack_fail(err.to_string()))
            .add_attribute("action", "ibc_packet_receive")
            .add_attribute("error", err.to_string())),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_ack(
    _deps: DepsMut,
    _env: Env,
    msg: IbcPacketAckMsg,
) -> Result<IbcBasicResponse, ContractError> {
    let ack: AckMsg = from_binary(&msg.acknowledgement.data)?;
    match ack {
        AckMsg::Ok(_) => Ok(IbcBasicResponse::new()
            .add_attribute("action", "ibc_packet_ack")
            .add_attribute("success", "true")),
        AckMsg::Error(err) => Ok(IbcBasicResponse::new()
            .add_attribute("action", "ibc_packet_ack")
            .add_attribute("success", "false")
            .add_attribute("error", err)),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_timeout(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketTimeoutMsg,
) -> Result<IbcBasicResponse, ContractError> {
    Ok(IbcBasicResponse::new()
        .add_attribute("action", "ibc_packet_timeout")
        .add_attribute("success", "false")
        .add_attribute("error", "timeout"))
}

fn validate_order_and_version(
    channel: &cosmwasm_std::IbcChannel,
    counterparty_version: Option<&str>,
) -> Result<(), ContractError> {
    if channel.order != IbcOrder::Unordered {
        return Err(ContractError::CrossChainError {
            msg: "Only unordered channels are supported".to_string(),
        });
    }

    if channel.version != IBC_VERSION {
        return Err(ContractError::CrossChainError {
            msg: format!("Must set version to `{}`", IBC_VERSION),
        });
    }

    if let Some(version) = counterparty_version {
        if version != IBC_VERSION {
            return Err(ContractError::CrossChainError {
                msg: format!("Counterparty version must be `{}`", IBC_VERSION),
            });
        }
    }

    Ok(())
}

fn process_received_payment(
    deps: DepsMut,
    env: Env,
    packet: CrossChainPaymentPacket,
) -> Result<String, ContractError> {
    use crate::state::{Payment, PaymentStatus, PAYMENTS, USER_PAYMENTS, PENDING_BALANCES, STATS};
    
    // Validate packet data
    if packet.amount.is_zero() {
        return Err(ContractError::InvalidAmount {});
    }

    // Generate payment ID for cross-chain payment
    let payment_id = format!(
        "ibc:{}:{}:{}:{}",
        packet.sender,
        packet.recipient,
        packet.amount,
        env.block.time.seconds()
    );

    let payment = Payment {
        id: payment_id.clone(),
        sender: packet.sender.clone(),
        recipient: packet.recipient.clone(),
        amount: packet.amount,
        token: packet.token,
        status: PaymentStatus::Completed, // Cross-chain payments are auto-completed
        proof_type: None,
        proof_data: packet.proof_data,
        description: packet.description,
        created_at: env.block.time.seconds(),
        completed_at: Some(env.block.time.seconds()),
        requires_proof: false,
    };

    PAYMENTS.save(deps.storage, &payment_id, &payment)?;

    // Update user payments
    let mut recipient_payments = USER_PAYMENTS
        .may_load(deps.storage, &packet.recipient)?
        .unwrap_or_default();
    recipient_payments.push(payment_id.clone());
    USER_PAYMENTS.save(deps.storage, &packet.recipient, &recipient_payments)?;

    // Update stats
    let mut stats = STATS.load(deps.storage)?;
    stats.total_payments += 1;
    stats.total_volume += packet.amount;
    STATS.save(deps.storage, &stats)?;

    Ok(payment_id)
}

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum AckMsg {
    Ok(String),
    Error(String),
}

fn ack_success() -> Binary {
    to_binary(&AckMsg::Ok("success".to_string())).unwrap()
}

fn ack_fail(err: String) -> Binary {
    to_binary(&AckMsg::Error(err)).unwrap()
}

// Helper function for binary deserialization
fn from_binary<T: serde::de::DeserializeOwned>(data: &Binary) -> StdResult<T> {
    cosmwasm_std::from_binary(data)
}