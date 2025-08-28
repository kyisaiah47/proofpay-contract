use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("User already registered")]
    UserAlreadyRegistered {},

    #[error("User not registered")]
    UserNotRegistered {},

    #[error("Username already taken")]
    UsernameTaken {},

    #[error("Invalid username")]
    InvalidUsername {},

    #[error("Payment not found")]
    PaymentNotFound {},

    #[error("Payment already completed")]
    PaymentAlreadyCompleted {},

    #[error("Payment not pending")]
    PaymentNotPending {},

    #[error("Insufficient funds")]
    InsufficientFunds {},

    #[error("Invalid amount")]
    InvalidAmount {},

    #[error("Invalid proof")]
    InvalidProof {},

    #[error("Proof required")]
    ProofRequired {},

    #[error("Cannot pay yourself")]
    CannotPaySelf {},

    #[error("Address already authorized")]
    AddressAlreadyAuthorized {},

    #[error("Address not authorized")]
    AddressNotAuthorized {},

    #[error("Invalid description length")]
    InvalidDescriptionLength {},

    #[error("Invalid dispute reason")]
    InvalidDisputeReason {},

    #[error("Cross-chain operation failed: {msg}")]
    CrossChainError { msg: String },
}