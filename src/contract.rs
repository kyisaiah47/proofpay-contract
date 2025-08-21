#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Order,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, JobResponse, JobsResponse};
use crate::state::{Job, JobStatus, State, JOBS, STATE};

const CONTRACT_NAME: &str = "crates.io:proof-of-work-contract";
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
        next_job_id: 1,
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
        ExecuteMsg::PostJob { description, deadline } => {
            execute_post_job(deps, env, info, description, deadline)
        }
        ExecuteMsg::AcceptJob { job_id } => execute_accept_job(deps, env, info, job_id),
        ExecuteMsg::SubmitProof { job_id, proof } => {
            execute_submit_proof(deps, env, info, job_id, proof)
        }
        ExecuteMsg::AcceptProof { job_id } => execute_accept_proof(deps, env, info, job_id),
        ExecuteMsg::RejectProof { job_id } => execute_reject_proof(deps, env, info, job_id),
        ExecuteMsg::CancelJob { job_id } => execute_cancel_job(deps, env, info, job_id),
    }
}

pub fn execute_post_job(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    description: String,
    deadline: Option<u64>,
) -> Result<Response, ContractError> {
    // Require payment attached
    if info.funds.is_empty() {
        return Err(ContractError::NoPaymentAttached {});
    }

    // Take the first coin as escrow amount
    let escrow_amount = info.funds[0].clone();
    
    // Validate payment amount > 0
    if escrow_amount.amount.is_zero() {
        return Err(ContractError::InvalidPaymentAmount {});
    }

    // Get and increment next job ID
    let mut state = STATE.load(deps.storage)?;
    let job_id = state.next_job_id;
    state.next_job_id += 1;
    STATE.save(deps.storage, &state)?;

    // Create new job
    let job = Job {
        id: job_id,
        client: info.sender.clone(),
        description: description.clone(),
        escrow_amount,
        worker: None,
        proof: None,
        status: JobStatus::Open,
        created_at: env.block.time.seconds(),
        deadline,
    };

    JOBS.save(deps.storage, job_id, &job)?;

    Ok(Response::new()
        .add_attribute("action", "post_job")
        .add_attribute("job_id", job_id.to_string())
        .add_attribute("client", info.sender.as_str())
        .add_attribute("description", description)
        .add_attribute("escrow_amount", job.escrow_amount.to_string()))
}

pub fn execute_accept_job(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    job_id: u64,
) -> Result<Response, ContractError> {
    JOBS.update(deps.storage, job_id, |job| -> Result<_, ContractError> {
        let mut job = job.ok_or(ContractError::JobNotFound {})?;

        // Check job status is Open
        if !matches!(job.status, JobStatus::Open) {
            return Err(ContractError::JobNotAvailable {});
        }

        // Verify sender is not the client
        if job.client == info.sender {
            return Err(ContractError::ClientCannotAcceptOwnJob {});
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

pub fn execute_submit_proof(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    job_id: u64,
    proof: String,
) -> Result<Response, ContractError> {
    JOBS.update(deps.storage, job_id, |job| -> Result<_, ContractError> {
        let mut job = job.ok_or(ContractError::JobNotFound {})?;

        // Check if worker is assigned and sender is the worker
        if let Some(ref worker) = job.worker {
            if *worker != info.sender {
                return Err(ContractError::NotAuthorized {});
            }
        } else {
            return Err(ContractError::NoWorkerAssigned {});
        }

        // Check job is in progress
        if !matches!(job.status, JobStatus::InProgress) {
            return Err(ContractError::JobNotInProgress {});
        }

        job.proof = Some(proof.clone());
        Ok(job)
    })?;

    Ok(Response::new()
        .add_attribute("action", "submit_proof")
        .add_attribute("job_id", job_id.to_string())
        .add_attribute("worker", info.sender.as_str())
        .add_attribute("proof", proof))
}

pub fn execute_accept_proof(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    job_id: u64,
) -> Result<Response, ContractError> {
    let job = JOBS.load(deps.storage, job_id)?;

    // Verify only client can accept
    if job.client != info.sender {
        return Err(ContractError::NotAuthorized {});
    }

    // Check proof exists and worker assigned
    if job.proof.is_none() {
        return Err(ContractError::NoProofSubmitted {});
    }

    let worker = job.worker.ok_or(ContractError::NoWorkerAssigned {})?;

    // Update job status to Completed
    JOBS.update(deps.storage, job_id, |job| -> Result<_, ContractError> {
        let mut job = job.ok_or(ContractError::JobNotFound {})?;
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
        .add_attribute("payment_sent_to", worker.as_str()))
}

pub fn execute_reject_proof(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    job_id: u64,
) -> Result<Response, ContractError> {
    JOBS.update(deps.storage, job_id, |job| -> Result<_, ContractError> {
        let mut job = job.ok_or(ContractError::JobNotFound {})?;

        // Verify only client can reject
        if job.client != info.sender {
            return Err(ContractError::NotAuthorized {});
        }

        // Check job is in progress with a submitted proof
        if !matches!(job.status, JobStatus::InProgress) {
            return Err(ContractError::JobNotInProgress {});
        }

        if job.proof.is_none() {
            return Err(ContractError::NoProofSubmitted {});
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

pub fn execute_cancel_job(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    job_id: u64,
) -> Result<Response, ContractError> {
    let job = JOBS.load(deps.storage, job_id)?;

    // Verify only client can cancel
    if job.client != info.sender {
        return Err(ContractError::NotAuthorized {});
    }

    // Check job not already completed
    if matches!(job.status, JobStatus::Completed) {
        return Err(ContractError::JobAlreadyCompleted {});
    }

    // Update job status to Cancelled
    JOBS.update(deps.storage, job_id, |job| -> Result<_, ContractError> {
        let mut job = job.ok_or(ContractError::JobNotFound {})?;
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
        .add_attribute("refund_sent_to", job.client.as_str()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetJob { job_id } => query_job(deps, job_id),
        QueryMsg::ListJobs {} => query_jobs(deps),
        QueryMsg::GetJobsByClient { client } => query_jobs_by_client(deps, client),
        QueryMsg::GetJobsByWorker { worker } => query_jobs_by_worker(deps, worker),
    }
}

fn query_job(deps: Deps, job_id: u64) -> StdResult<Binary> {
    let job = JOBS.load(deps.storage, job_id)?;
    to_json_binary(&JobResponse { job })
}

fn query_jobs(deps: Deps) -> StdResult<Binary> {
    let jobs: StdResult<Vec<Job>> = JOBS
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| item.map(|(_, job)| job))
        .collect();
    to_json_binary(&JobsResponse { jobs: jobs? })
}

fn query_jobs_by_client(deps: Deps, client: String) -> StdResult<Binary> {
    let client_addr = deps.api.addr_validate(&client)?;
    let jobs: StdResult<Vec<Job>> = JOBS
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| item.map(|(_, job)| job))
        .filter(|job| {
            job.as_ref()
                .map(|j| j.client == client_addr)
                .unwrap_or(false)
        })
        .collect();
    to_json_binary(&JobsResponse { jobs: jobs? })
}

fn query_jobs_by_worker(deps: Deps, worker: String) -> StdResult<Binary> {
    let worker_addr = deps.api.addr_validate(&worker)?;
    let jobs: StdResult<Vec<Job>> = JOBS
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| item.map(|(_, job)| job))
        .filter(|job| {
            job.as_ref()
                .map(|j| j.worker.as_ref() == Some(&worker_addr))
                .unwrap_or(false)
        })
        .collect();
    to_json_binary(&JobsResponse { jobs: jobs? })
}
