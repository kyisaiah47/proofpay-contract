use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
    
    // User Management Errors
    #[error("Username already taken")]
    UsernameAlreadyTaken {},
    
    #[error("User not found")]
    UserNotFound {},
    
    #[error("Invalid username format")]
    InvalidUsername {},
    
    #[error("Wallet already registered")]
    WalletAlreadyRegistered {},
    
    #[error("User not registered")]
    UserNotRegistered {},
    
    // Friends System Errors
    #[error("Cannot send friend request to yourself")]
    CannotAddSelf {},
    
    #[error("Friend request already exists")]
    FriendRequestAlreadyExists {},
    
    #[error("Friend request not found")]
    FriendRequestNotFound {},
    
    #[error("Users are already friends")]
    AlreadyFriends {},
    
    #[error("Users are not friends")]
    NotFriends {},
    
    #[error("Cannot send friend request to non-friend")]
    CannotRequestNonFriend {},
    
    // Payment System Errors
    #[error("Payment not found")]
    PaymentNotFound {},
    
    #[error("Not authorized to access this payment")]
    PaymentNotAuthorized {},
    
    #[error("Payment already completed")]
    PaymentAlreadyCompleted {},
    
    #[error("Payment already cancelled")]
    PaymentAlreadyCancelled {},
    
    #[error("Cannot send payment to yourself")]
    CannotPaySelf {},
    
    #[error("Insufficient funds")]
    InsufficientFunds {},
    
    #[error("Invalid payment amount")]
    InvalidPaymentAmount {},
    
    #[error("Proof already submitted")]
    ProofAlreadySubmitted {},
    
    #[error("No proof required for this payment")]
    NoProofRequired {},
    
    #[error("Proof required before approval")]
    ProofRequired {},
    
    #[error("Invalid proof type")]
    InvalidProofType {},
    
    // Authorization Errors
    #[error("Not authorized")]
    NotAuthorized {},
    
    #[error("Only payment sender can cancel")]
    OnlySenderCanCancel {},
    
    #[error("Only payment recipient can approve")]
    OnlyRecipientCanApprove {},
    
    // Task System Errors
    #[error("Task not found")]
    TaskNotFound {},
    
    #[error("Not authorized to access this task")]
    TaskNotAuthorized {},
    
    #[error("Task already completed")]
    TaskAlreadyCompleted {},
    
    #[error("Task already disputed")]
    TaskAlreadyDisputed {},
    
    #[error("Task deadline expired")]
    TaskExpired {},
    
    #[error("Task not in dispute")]
    TaskNotInDispute {},
    
    #[error("Dispute window has not elapsed")]
    DisputeWindowNotElapsed {},
    
    #[error("Invalid proof")]
    InvalidProof {},
    
    #[error("zkTLS verification failed")]
    ZkTlsVerificationFailed {},
    
    #[error("Only payer can approve soft tasks")]
    OnlyPayerCanApproveSoft {},
    
    #[error("Only payer can dispute tasks")]
    OnlyPayerCanDispute {},
    
    #[error("Only owner can resolve disputes")]
    OnlyOwnerCanResolveDispute {},
    
    #[error("Cannot create task with yourself")]
    CannotCreateTaskWithSelf {},
    
    #[error("Invalid task deadline")]
    InvalidTaskDeadline {},
}
