#[cfg(test)]
mod tests {
    use crate::helpers::ProofOfWorkContract;
    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
    use crate::state::JobStatus;
    use cosmwasm_std::{Addr, Coin, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    pub fn contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }

    const CLIENT: &str = "client1";
    const WORKER: &str = "worker1";
    const ADMIN: &str = "admin";
    const NATIVE_DENOM: &str = "uxion";

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(CLIENT),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(1000),
                    }],
                )
                .unwrap();
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(WORKER),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(100),
                    }],
                )
                .unwrap();
        })
    }

    fn proper_instantiate() -> (App, ProofOfWorkContract) {
        let mut app = mock_app();
        let contract_id = app.store_code(contract_template());

        let msg = InstantiateMsg {};
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "proof-of-work",
                None,
            )
            .unwrap();

        let contract = ProofOfWorkContract(contract_addr);

        (app, contract)
    }

    mod job_lifecycle {
        use super::*;

        #[test]
        fn test_post_job() {
            let (mut app, contract) = proper_instantiate();

            // Post a job with payment
            let payment = vec![Coin {
                denom: NATIVE_DENOM.to_string(),
                amount: Uint128::new(100),
            }];

            let msg = ExecuteMsg::PostJob {
                description: "Test job description".to_string(),
                deadline: None,
            };

            app.execute_contract(
                Addr::unchecked(CLIENT),
                contract.addr(),
                &msg,
                &payment,
            )
            .unwrap();

            // Query the job
            let job_response: crate::msg::JobResponse = app
                .wrap()
                .query_wasm_smart(contract.addr(), &QueryMsg::GetJob { job_id: 1 })
                .unwrap();

            assert_eq!(job_response.job.id, 1);
            assert_eq!(job_response.job.client, Addr::unchecked(CLIENT));
            assert_eq!(job_response.job.description, "Test job description");
            assert_eq!(job_response.job.status, JobStatus::Open);
            assert_eq!(job_response.job.escrow_amount.amount, Uint128::new(100));
        }

        #[test]
        fn test_accept_job() {
            let (mut app, contract) = proper_instantiate();

            // Post a job
            let payment = vec![Coin {
                denom: NATIVE_DENOM.to_string(),
                amount: Uint128::new(100),
            }];

            let post_msg = ExecuteMsg::PostJob {
                description: "Test job".to_string(),
                deadline: None,
            };

            app.execute_contract(
                Addr::unchecked(CLIENT),
                contract.addr(),
                &post_msg,
                &payment,
            )
            .unwrap();

            // Worker accepts the job
            let accept_msg = ExecuteMsg::AcceptJob { job_id: 1 };

            app.execute_contract(Addr::unchecked(WORKER), contract.addr(), &accept_msg, &[])
                .unwrap();

            // Query the job
            let job_response: crate::msg::JobResponse = app
                .wrap()
                .query_wasm_smart(contract.addr(), &QueryMsg::GetJob { job_id: 1 })
                .unwrap();

            assert_eq!(job_response.job.status, JobStatus::InProgress);
            assert_eq!(job_response.job.worker, Some(Addr::unchecked(WORKER)));
        }

        #[test]
        fn test_submit_and_accept_proof() {
            let (mut app, contract) = proper_instantiate();

            // Post a job
            let payment = vec![Coin {
                denom: NATIVE_DENOM.to_string(),
                amount: Uint128::new(100),
            }];

            let post_msg = ExecuteMsg::PostJob {
                description: "Test job".to_string(),
                deadline: None,
            };

            app.execute_contract(
                Addr::unchecked(CLIENT),
                contract.addr(),
                &post_msg,
                &payment,
            )
            .unwrap();

            // Worker accepts the job
            let accept_msg = ExecuteMsg::AcceptJob { job_id: 1 };
            app.execute_contract(Addr::unchecked(WORKER), contract.addr(), &accept_msg, &[])
                .unwrap();

            // Worker submits proof
            let submit_msg = ExecuteMsg::SubmitProof {
                job_id: 1,
                proof: "Proof of work completed".to_string(),
            };
            app.execute_contract(Addr::unchecked(WORKER), contract.addr(), &submit_msg, &[])
                .unwrap();

            // Client accepts proof
            let accept_proof_msg = ExecuteMsg::AcceptProof { job_id: 1 };
            app.execute_contract(
                Addr::unchecked(CLIENT),
                contract.addr(),
                &accept_proof_msg,
                &[],
            )
            .unwrap();

            // Query the job
            let job_response: crate::msg::JobResponse = app
                .wrap()
                .query_wasm_smart(contract.addr(), &QueryMsg::GetJob { job_id: 1 })
                .unwrap();

            assert_eq!(job_response.job.status, JobStatus::Completed);
            assert_eq!(
                job_response.job.proof,
                Some("Proof of work completed".to_string())
            );

            // Check that worker received payment
            let worker_balance = app.wrap().query_balance(WORKER, NATIVE_DENOM).unwrap();
            assert_eq!(worker_balance.amount, Uint128::new(200)); // 100 initial + 100 payment
        }

        #[test]
        fn test_cancel_job() {
            let (mut app, contract) = proper_instantiate();

            // Post a job
            let payment = vec![Coin {
                denom: NATIVE_DENOM.to_string(),
                amount: Uint128::new(100),
            }];

            let post_msg = ExecuteMsg::PostJob {
                description: "Test job".to_string(),
                deadline: None,
            };

            app.execute_contract(
                Addr::unchecked(CLIENT),
                contract.addr(),
                &post_msg,
                &payment,
            )
            .unwrap();

            // Client cancels the job
            let cancel_msg = ExecuteMsg::CancelJob { job_id: 1 };
            app.execute_contract(Addr::unchecked(CLIENT), contract.addr(), &cancel_msg, &[])
                .unwrap();

            // Query the job
            let job_response: crate::msg::JobResponse = app
                .wrap()
                .query_wasm_smart(contract.addr(), &QueryMsg::GetJob { job_id: 1 })
                .unwrap();

            assert_eq!(job_response.job.status, JobStatus::Cancelled);

            // Check that client received refund
            let client_balance = app.wrap().query_balance(CLIENT, NATIVE_DENOM).unwrap();
            assert_eq!(client_balance.amount, Uint128::new(1000)); // Full refund
        }
    }

    mod error_cases {
        use super::*;

        #[test]
        fn test_client_cannot_accept_own_job() {
            let (mut app, contract) = proper_instantiate();

            // Post a job
            let payment = vec![Coin {
                denom: NATIVE_DENOM.to_string(),
                amount: Uint128::new(100),
            }];

            let post_msg = ExecuteMsg::PostJob {
                description: "Test job".to_string(),
                deadline: None,
            };

            app.execute_contract(
                Addr::unchecked(CLIENT),
                contract.addr(),
                &post_msg,
                &payment,
            )
            .unwrap();

            // Client tries to accept their own job (should fail)
            let accept_msg = ExecuteMsg::AcceptJob { job_id: 1 };
            let result = app.execute_contract(
                Addr::unchecked(CLIENT),
                contract.addr(),
                &accept_msg,
                &[],
            );

            assert!(result.is_err());
        }

        #[test]
        fn test_post_job_without_payment_fails() {
            let (mut app, contract) = proper_instantiate();

            // Try to post a job without payment (should fail)
            let post_msg = ExecuteMsg::PostJob {
                description: "Test job".to_string(),
                deadline: None,
            };

            let result = app.execute_contract(
                Addr::unchecked(CLIENT),
                contract.addr(),
                &post_msg,
                &[],
            );

            assert!(result.is_err());
        }
    }
}
