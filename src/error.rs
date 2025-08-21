use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
    
    #[error("Unauthorized")]
    Unauthorized {},
    
    #[error("NotFound")]
    NotFound {},
    
    #[error("No payment attached")]
    NoPaymentAttached,
    
    #[error("Invalid payment amount")]
    InvalidPaymentAmount,
    
    #[error("Job not available")]
    JobNotAvailable,
    
    #[error("Client cannot accept own job")]
    ClientCannotAcceptOwnJob,
    
    #[error("Job not in progress")]
    JobNotInProgress,
    
    #[error("No proof submitted")]
    NoProofSubmitted,
    
    #[error("No worker assigned")]
    NoWorkerAssigned,
    
    #[error("Job already completed")]
    JobAlreadyCompleted,
    
    #[error("Not authorized")]
    NotAuthorized,
}
