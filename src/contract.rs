#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, JobResponse, JobsResponse};
use crate::state::{Job, JobStatus, JOBS, JOB_ID, State, STATE};

const CONTRACT_NAME: &str = "crates.io:proof-of-work";
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
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;
    JOB_ID.save(deps.storage, &0)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::PostJob { description, deadline } => post_job(deps, env, info, description, deadline),
        ExecuteMsg::AcceptJob { job_id } => accept_job(deps, env, info, job_id),
        ExecuteMsg::SubmitProof { job_id, proof } => submit_proof(deps, env, info, job_id, proof),
        ExecuteMsg::AcceptProof { job_id } => accept_proof(deps, env, info, job_id),
        ExecuteMsg::RejectProof { job_id } => reject_proof(deps, env, info, job_id),
        ExecuteMsg::CancelJob { job_id } => cancel_job(deps, env, info, job_id),
    }
}

pub fn post_job(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    description: String,
    deadline: Option<u64>,
) -> Result<Response, ContractError> {
    // Require payment attached
    if info.funds.is_empty() {
        return Err(ContractError::NoPaymentAttached);
    }
    
    // For simplicity, we'll take the first coin as escrow
    let escrow_amount = info.funds[0].clone();
    
    // Validate payment amount > 0
    if escrow_amount.amount.is_zero() {
        return Err(ContractError::InvalidPaymentAmount);
    }

    let mut id = JOB_ID.load(deps.storage)?;
    id += 1;
    JOB_ID.save(deps.storage, &id)?;

    let job = Job {
        id,
        client: info.sender.clone(),
        worker: None,
        description,
        proof: None,
        accepted: false,
        escrow_amount,
        status: JobStatus::Open,
        deadline,
    };
    
    JOBS.save(deps.storage, id, &job)?;
    
    Ok(Response::new()
        .add_attribute("action", "post_job")
        .add_attribute("job_id", id.to_string())
        .add_attribute("escrow_amount", job.escrow_amount.to_string()))
}

pub fn accept_job(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    job_id: u64,
) -> Result<Response, ContractError> {
    JOBS.update(deps.storage, job_id, |job| -> Result<_, ContractError> {
        let mut job = job.ok_or(ContractError::NotFound {})?;
        
        // Check job status is Open
        if !matches!(job.status, JobStatus::Open) {
            return Err(ContractError::JobNotAvailable);
        }
        
        // Verify sender is not the client
        if job.client == info.sender {
            return Err(ContractError::ClientCannotAcceptOwnJob);
        }
        
        // Assign worker and set status to InProgress
        job.worker = Some(info.sender.clone());
        job.status = JobStatus::InProgress;
        
        Ok(job)
    })?;
    
    Ok(Response::new()
        .add_attribute("action", "accept_job")
        .add_attribute("job_id", job_id.to_string())
        .add_attribute("worker", info.sender.as_str()))
}

pub fn submit_proof(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    job_id: u64,
    proof: String,
) -> Result<Response, ContractError> {
    JOBS.update(deps.storage, job_id, |job| -> Result<_, ContractError> {
        let mut job = job.ok_or(ContractError::NotFound {})?;
        
        // Check if worker is assigned and sender is the worker
        if let Some(ref worker) = job.worker {
            if *worker != info.sender {
                return Err(ContractError::NotAuthorized);
            }
        } else {
            return Err(ContractError::NoWorkerAssigned);
        }
        
        // Check job is in progress
        if !matches!(job.status, JobStatus::InProgress) {
            return Err(ContractError::JobNotInProgress);
        }
        
        job.proof = Some(proof);
        Ok(job)
    })?;
    
    Ok(Response::new()
        .add_attribute("action", "submit_proof")
        .add_attribute("job_id", job_id.to_string()))
}

pub fn accept_proof(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    job_id: u64,
) -> Result<Response, ContractError> {
    let job = JOBS.load(deps.storage, job_id)?;
    
    // Verify only client can accept
    if job.client != info.sender {
        return Err(ContractError::NotAuthorized);
    }
    
    // Check proof exists and worker assigned
    if job.proof.is_none() {
        return Err(ContractError::NoProofSubmitted);
    }
    
    let worker = job.worker.ok_or(ContractError::NoWorkerAssigned)?;
    
    // Update job status to Completed
    JOBS.update(deps.storage, job_id, |job| -> Result<_, ContractError> {
        let mut job = job.ok_or(ContractError::NotFound {})?;
        job.accepted = true;
        job.status = JobStatus::Completed;
        Ok(job)
    })?;
    
    // Send payment to worker
    let payment_msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: worker.to_string(),
        amount: vec![job.escrow_amount],
    });
    
    Ok(Response::new()
        .add_message(payment_msg)
        .add_attribute("action", "accept_proof")
        .add_attribute("job_id", job_id.to_string())
        .add_attribute("payment_sent", worker.as_str()))
}

pub fn reject_proof(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    job_id: u64,
) -> Result<Response, ContractError> {
    JOBS.update(deps.storage, job_id, |job| -> Result<_, ContractError> {
        let mut job = job.ok_or(ContractError::NotFound {})?;
        
        // Verify only client can reject
        if job.client != info.sender {
            return Err(ContractError::NotAuthorized);
        }
        
        // Check job is in progress with a submitted proof
        if !matches!(job.status, JobStatus::InProgress) {
            return Err(ContractError::JobNotInProgress);
        }
        
        if job.proof.is_none() {
            return Err(ContractError::NoProofSubmitted);
        }
        
        // Clear proof but keep job InProgress
        job.proof = None;
        job.status = JobStatus::InProgress;
        
        Ok(job)
    })?;
    
    Ok(Response::new()
        .add_attribute("action", "reject_proof")
        .add_attribute("job_id", job_id.to_string()))
}

pub fn cancel_job(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    job_id: u64,
) -> Result<Response, ContractError> {
    let job = JOBS.load(deps.storage, job_id)?;
    
    // Verify only client can cancel
    if job.client != info.sender {
        return Err(ContractError::NotAuthorized);
    }
    
    // Check job not already completed
    if matches!(job.status, JobStatus::Completed) {
        return Err(ContractError::JobAlreadyCompleted);
    }
    
    // Update job status to Cancelled
    JOBS.update(deps.storage, job_id, |job| -> Result<_, ContractError> {
        let mut job = job.ok_or(ContractError::NotFound {})?;
        job.status = JobStatus::Cancelled;
        Ok(job)
    })?;
    
    // Refund escrow to client
    let refund_msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: job.client.to_string(),
        amount: vec![job.escrow_amount],
    });
    
    Ok(Response::new()
        .add_message(refund_msg)
        .add_attribute("action", "cancel_job")
        .add_attribute("job_id", job_id.to_string())
        .add_attribute("refund_sent", job.client.as_str()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetJob { job_id } => {
            let job = JOBS.load(deps.storage, job_id)?;
            to_json_binary(&JobResponse { job })
        }
        QueryMsg::ListJobs {} => {
            let jobs: StdResult<Vec<Job>> = JOBS
                .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
                .map(|item| item.map(|(_, job)| job))
                .collect();
            to_json_binary(&JobsResponse { jobs: jobs? })
        }
    }
}
