#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, JobResponse, JobsResponse};
use crate::state::{Job, JOBS, JOB_ID, State, STATE};

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
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::PostJob { description } => {
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
            };
            JOBS.save(deps.storage, id, &job)?;
            Ok(Response::new().add_attribute("action", "post_job").add_attribute("job_id", id.to_string()))
        }
        ExecuteMsg::SubmitProof { job_id, proof } => {
            JOBS.update(deps.storage, job_id, |job| -> Result<_, ContractError> {
                let mut job = job.ok_or(ContractError::NotFound {})?;
                if job.worker.is_none() {
                    job.worker = Some(info.sender.clone());
                }
                job.proof = Some(proof);
                Ok(job)
            })?;
            Ok(Response::new().add_attribute("action", "submit_proof").add_attribute("job_id", job_id.to_string()))
        }
        ExecuteMsg::AcceptProof { job_id } => {
            JOBS.update(deps.storage, job_id, |job| -> Result<_, ContractError> {
                let mut job = job.ok_or(ContractError::NotFound {})?;
                if job.client != info.sender {
                    return Err(ContractError::Unauthorized {});
                }
                job.accepted = true;
                Ok(job)
            })?;
            Ok(Response::new().add_attribute("action", "accept_proof").add_attribute("job_id", job_id.to_string()))
        }
    }
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
