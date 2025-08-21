use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
    
    #[error("Job not found")]
    JobNotFound {},
    
    #[error("No payment attached")]
    NoPaymentAttached {},
    
    #[error("Invalid payment amount")]
    InvalidPaymentAmount {},
    
    #[error("Job not available")]
    JobNotAvailable {},
    
    #[error("Not authorized")]
    NotAuthorized {},
    
    #[error("Job not in progress")]
    JobNotInProgress {},
    
    #[error("No proof submitted")]
    NoProofSubmitted {},
    
    #[error("Client cannot accept own job")]
    ClientCannotAcceptOwnJob {},
    
    #[error("Job already completed")]
    JobAlreadyCompleted {},
    
    #[error("No worker assigned")]
    NoWorkerAssigned {},
}
