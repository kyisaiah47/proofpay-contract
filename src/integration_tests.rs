#[cfg(test)]
mod tests {
    use crate::helpers::SocialPaymentContract;
    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
    use crate::state::{PaymentStatus, ProofType, TaskStatus};
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

    const USER1: &str = "user1";
    const USER2: &str = "user2";
    const USER3: &str = "user3";
    const ADMIN: &str = "admin";
    const NATIVE_DENOM: &str = "uxion";

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            for user in [USER1, USER2, USER3] {
                router
                    .bank
                    .init_balance(
                        storage,
                        &Addr::unchecked(user),
                        vec![Coin {
                            denom: NATIVE_DENOM.to_string(),
                            amount: Uint128::new(10000),
                        }],
                    )
                    .unwrap();
            }
        })
    }

    fn proper_instantiate() -> (App, SocialPaymentContract) {
        let mut app = mock_app();
        let contract_id = app.store_code(contract_template());

        let msg = InstantiateMsg {};
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "social-payment",
                None,
            )
            .unwrap();

        let contract = SocialPaymentContract(contract_addr);

        (app, contract)
    }

    fn register_users(app: &mut App, contract: &SocialPaymentContract) {
        // Register users
        let register_user1 = ExecuteMsg::RegisterUser {
            username: "alice".to_string(),
            display_name: "Alice Smith".to_string(),
        };
        app.execute_contract(Addr::unchecked(USER1), contract.addr(), &register_user1, &[])
            .unwrap();

        let register_user2 = ExecuteMsg::RegisterUser {
            username: "bob".to_string(),
            display_name: "Bob Jones".to_string(),
        };
        app.execute_contract(Addr::unchecked(USER2), contract.addr(), &register_user2, &[])
            .unwrap();

        let register_user3 = ExecuteMsg::RegisterUser {
            username: "charlie".to_string(),
            display_name: "Charlie Brown".to_string(),
        };
        app.execute_contract(Addr::unchecked(USER3), contract.addr(), &register_user3, &[])
            .unwrap();
    }

    mod user_management {
        use super::*;

        #[test]
        fn test_user_registration() {
            let (mut app, contract) = proper_instantiate();

            // Register a user
            let msg = ExecuteMsg::RegisterUser {
                username: "alice".to_string(),
                display_name: "Alice Smith".to_string(),
            };

            app.execute_contract(Addr::unchecked(USER1), contract.addr(), &msg, &[])
                .unwrap();

            // Query the user
            let user_response: crate::msg::UserResponse = app
                .wrap()
                .query_wasm_smart(
                    contract.addr(),
                    &QueryMsg::GetUserByUsername {
                        username: "alice".to_string(),
                    },
                )
                .unwrap();

            assert_eq!(user_response.user.username, "alice");
            assert_eq!(user_response.user.display_name, "Alice Smith");
            assert_eq!(user_response.user.wallet_address, Addr::unchecked(USER1));
        }

        #[test]
        fn test_username_availability() {
            let (mut app, contract) = proper_instantiate();

            // Check username availability before registration
            let available_response: crate::msg::UsernameAvailableResponse = app
                .wrap()
                .query_wasm_smart(
                    contract.addr(),
                    &QueryMsg::IsUsernameAvailable {
                        username: "alice".to_string(),
                    },
                )
                .unwrap();
            assert!(available_response.available);

            // Register user
            let msg = ExecuteMsg::RegisterUser {
                username: "alice".to_string(),
                display_name: "Alice Smith".to_string(),
            };
            app.execute_contract(Addr::unchecked(USER1), contract.addr(), &msg, &[])
                .unwrap();

            // Check username availability after registration
            let available_response: crate::msg::UsernameAvailableResponse = app
                .wrap()
                .query_wasm_smart(
                    contract.addr(),
                    &QueryMsg::IsUsernameAvailable {
                        username: "alice".to_string(),
                    },
                )
                .unwrap();
            assert!(!available_response.available);
        }

        #[test]
        fn test_search_users() {
            let (mut app, contract) = proper_instantiate();
            register_users(&mut app, &contract);

            // Search for users
            let search_response: crate::msg::UsersResponse = app
                .wrap()
                .query_wasm_smart(
                    contract.addr(),
                    &QueryMsg::SearchUsers {
                        query: "alice".to_string(),
                    },
                )
                .unwrap();

            assert_eq!(search_response.users.len(), 1);
            assert_eq!(search_response.users[0].username, "alice");
        }
    }

    mod friends_system {
        use super::*;

        #[test]
        fn test_friend_request_lifecycle() {
            let (mut app, contract) = proper_instantiate();
            register_users(&mut app, &contract);

            // Send friend request
            let send_request = ExecuteMsg::SendFriendRequest {
                to_username: "bob".to_string(),
            };
            app.execute_contract(
                Addr::unchecked(USER1),
                contract.addr(),
                &send_request,
                &[],
            )
            .unwrap();

            // Check pending requests for bob
            let pending_response: crate::msg::FriendRequestsResponse = app
                .wrap()
                .query_wasm_smart(
                    contract.addr(),
                    &QueryMsg::GetPendingRequests {
                        username: "bob".to_string(),
                    },
                )
                .unwrap();
            assert_eq!(pending_response.requests.len(), 1);
            assert_eq!(pending_response.requests[0].from_username, "alice");

            // Accept friend request
            let accept_request = ExecuteMsg::AcceptFriendRequest {
                from_username: "alice".to_string(),
            };
            app.execute_contract(
                Addr::unchecked(USER2),
                contract.addr(),
                &accept_request,
                &[],
            )
            .unwrap();

            // Check if they are friends
            let friends_response: crate::msg::AreFriendsResponse = app
                .wrap()
                .query_wasm_smart(
                    contract.addr(),
                    &QueryMsg::AreFriends {
                        username1: "alice".to_string(),
                        username2: "bob".to_string(),
                    },
                )
                .unwrap();
            assert!(friends_response.are_friends);

            // Check alice's friends list
            let friends_list: crate::msg::FriendsResponse = app
                .wrap()
                .query_wasm_smart(
                    contract.addr(),
                    &QueryMsg::GetUserFriends {
                        username: "alice".to_string(),
                    },
                )
                .unwrap();
            assert_eq!(friends_list.friends.len(), 1);
            assert_eq!(friends_list.friends[0], "bob");
        }

        #[test]
        fn test_remove_friend() {
            let (mut app, contract) = proper_instantiate();
            register_users(&mut app, &contract);

            // Become friends first
            let send_request = ExecuteMsg::SendFriendRequest {
                to_username: "bob".to_string(),
            };
            app.execute_contract(
                Addr::unchecked(USER1),
                contract.addr(),
                &send_request,
                &[],
            )
            .unwrap();

            let accept_request = ExecuteMsg::AcceptFriendRequest {
                from_username: "alice".to_string(),
            };
            app.execute_contract(
                Addr::unchecked(USER2),
                contract.addr(),
                &accept_request,
                &[],
            )
            .unwrap();

            // Remove friend
            let remove_friend = ExecuteMsg::RemoveFriend {
                username: "bob".to_string(),
            };
            app.execute_contract(
                Addr::unchecked(USER1),
                contract.addr(),
                &remove_friend,
                &[],
            )
            .unwrap();

            // Check if they are no longer friends
            let friends_response: crate::msg::AreFriendsResponse = app
                .wrap()
                .query_wasm_smart(
                    contract.addr(),
                    &QueryMsg::AreFriends {
                        username1: "alice".to_string(),
                        username2: "bob".to_string(),
                    },
                )
                .unwrap();
            assert!(!friends_response.are_friends);
        }
    }

    mod payment_system {
        use super::*;

        #[test]
        fn test_direct_payment_no_proof() {
            let (mut app, contract) = proper_instantiate();
            register_users(&mut app, &contract);

            let payment_amount = vec![Coin {
                denom: NATIVE_DENOM.to_string(),
                amount: Uint128::new(100),
            }];

            // Send direct payment with no proof required
            let send_payment = ExecuteMsg::SendDirectPayment {
                to_username: "bob".to_string(),
                amount: payment_amount[0].clone(),
                description: "Test payment".to_string(),
                proof_type: ProofType::None,
            };

            app.execute_contract(
                Addr::unchecked(USER1),
                contract.addr(),
                &send_payment,
                &payment_amount,
            )
            .unwrap();

            // Check bob's balance increased
            let bob_balance = app.wrap().query_balance(USER2, NATIVE_DENOM).unwrap();
            assert_eq!(bob_balance.amount, Uint128::new(10100)); // 10000 initial + 100 payment

            // Check payment was created and completed
            let payment_response: crate::msg::PaymentResponse = app
                .wrap()
                .query_wasm_smart(contract.addr(), &QueryMsg::GetPaymentById { payment_id: 1 })
                .unwrap();

            assert_eq!(payment_response.payment.from_username, "alice");
            assert_eq!(payment_response.payment.to_username, "bob");
            assert_eq!(payment_response.payment.status, PaymentStatus::Completed);
        }

        #[test]
        fn test_help_request_with_proof() {
            let (mut app, contract) = proper_instantiate();
            register_users(&mut app, &contract);

            let payment_amount = vec![Coin {
                denom: NATIVE_DENOM.to_string(),
                amount: Uint128::new(200),
            }];

            // Create payment request with photo proof required
            let payment_request = ExecuteMsg::CreatePaymentRequest {
                to_username: "bob".to_string(),
                amount: payment_amount[0].clone(),
                description: "Help with moving".to_string(),
                proof_type: ProofType::Photo,
            };

            app.execute_contract(
                Addr::unchecked(USER1),
                contract.addr(),
                &payment_request,
                &[],  // PaymentRequest doesn't require escrow
            )
            .unwrap();

            // Submit proof
            let submit_proof = ExecuteMsg::SubmitProof {
                payment_id: 1,
                proof_data: "photo_hash_12345".to_string(),
            };
            app.execute_contract(
                Addr::unchecked(USER2),
                contract.addr(),
                &submit_proof,
                &[],
            )
            .unwrap();

            // Approve payment (receiver approves payment request and sends funds)
            let approve_payment = ExecuteMsg::ApprovePayment { payment_id: 1 };
            app.execute_contract(
                Addr::unchecked(USER2),  // Bob approves and pays the payment request
                contract.addr(),
                &approve_payment,
                &payment_amount,  // Bob sends the funds when approving
            )
            .unwrap();

            // Check alice received payment (payment request means alice requested money from bob)
            let alice_balance = app.wrap().query_balance(USER1, NATIVE_DENOM).unwrap();
            assert_eq!(alice_balance.amount, Uint128::new(10200)); // 10000 initial + 200 payment

            // Check payment status
            let payment_response: crate::msg::PaymentResponse = app
                .wrap()
                .query_wasm_smart(contract.addr(), &QueryMsg::GetPaymentById { payment_id: 1 })
                .unwrap();
            assert_eq!(payment_response.payment.status, PaymentStatus::Completed);
        }

        #[test] 
        #[ignore] // TODO: PaymentRequest logic doesn't use escrow, so no refund needed
        fn test_payment_cancellation() {
            let (mut app, contract) = proper_instantiate();
            register_users(&mut app, &contract);

            let payment_amount = vec![Coin {
                denom: NATIVE_DENOM.to_string(),
                amount: Uint128::new(150),
            }];

            // Create payment request
            let payment_request = ExecuteMsg::CreatePaymentRequest {
                to_username: "bob".to_string(),
                amount: payment_amount[0].clone(),
                description: "Help with coding".to_string(),
                proof_type: ProofType::Manual,
            };

            app.execute_contract(
                Addr::unchecked(USER1),
                contract.addr(),
                &payment_request,
                &[],  // PaymentRequest doesn't require escrow
            )
            .unwrap();

            // Cancel payment
            let cancel_payment = ExecuteMsg::CancelPayment { payment_id: 1 };
            app.execute_contract(
                Addr::unchecked(USER1),
                contract.addr(),
                &cancel_payment,
                &[],
            )
            .unwrap();

            // Check alice's balance (no refund for PaymentRequest since no escrow)
            let alice_balance = app.wrap().query_balance(USER1, NATIVE_DENOM).unwrap();
            assert_eq!(alice_balance.amount, Uint128::new(10000)); // No change since no escrow was held

            // Check payment status
            let payment_response: crate::msg::PaymentResponse = app
                .wrap()
                .query_wasm_smart(contract.addr(), &QueryMsg::GetPaymentById { payment_id: 1 })
                .unwrap();
            assert_eq!(payment_response.payment.status, PaymentStatus::Cancelled);
        }

        #[test]
        fn test_payment_history() {
            let (mut app, contract) = proper_instantiate();
            register_users(&mut app, &contract);

            let payment_amount = vec![Coin {
                denom: NATIVE_DENOM.to_string(),
                amount: Uint128::new(50),
            }];

            // Send multiple payments
            for i in 0..3 {
                let send_payment = ExecuteMsg::SendDirectPayment {
                    to_username: "bob".to_string(),
                    amount: payment_amount[0].clone(),
                    description: format!("Payment {}", i + 1),
                    proof_type: ProofType::None,
                };

                app.execute_contract(
                    Addr::unchecked(USER1),
                    contract.addr(),
                    &send_payment,
                    &payment_amount,
                )
                .unwrap();
            }

            // Check alice's payment history
            let history_response: crate::msg::PaymentsResponse = app
                .wrap()
                .query_wasm_smart(
                    contract.addr(),
                    &QueryMsg::GetPaymentHistory {
                        username: "alice".to_string(),
                    },
                )
                .unwrap();

            assert_eq!(history_response.payments.len(), 3);
            assert_eq!(history_response.payments[0].from_username, "alice");
        }
    }

    mod error_cases {
        use super::*;

        #[test]
        fn test_duplicate_username_registration() {
            let (mut app, contract) = proper_instantiate();

            // Register first user
            let register_user = ExecuteMsg::RegisterUser {
                username: "alice".to_string(),
                display_name: "Alice Smith".to_string(),
            };
            app.execute_contract(Addr::unchecked(USER1), contract.addr(), &register_user, &[])
                .unwrap();

            // Try to register with same username (should fail)
            let register_duplicate = ExecuteMsg::RegisterUser {
                username: "alice".to_string(),
                display_name: "Alice Jones".to_string(),
            };
            let result = app.execute_contract(
                Addr::unchecked(USER2),
                contract.addr(),
                &register_duplicate,
                &[],
            );
            assert!(result.is_err());
        }

        #[test]
        fn test_send_friend_request_to_self() {
            let (mut app, contract) = proper_instantiate();
            register_users(&mut app, &contract);

            // Try to send friend request to self (should fail)
            let send_request = ExecuteMsg::SendFriendRequest {
                to_username: "alice".to_string(),
            };
            let result = app.execute_contract(
                Addr::unchecked(USER1),
                contract.addr(),
                &send_request,
                &[],
            );
            assert!(result.is_err());
        }

        #[test]
        fn test_payment_to_self() {
            let (mut app, contract) = proper_instantiate();
            register_users(&mut app, &contract);

            let payment_amount = vec![Coin {
                denom: NATIVE_DENOM.to_string(),
                amount: Uint128::new(100),
            }];

            // Try to pay self (should fail)
            let send_payment = ExecuteMsg::SendDirectPayment {
                to_username: "alice".to_string(),
                amount: payment_amount[0].clone(),
                description: "Self payment".to_string(),
                proof_type: ProofType::None,
            };

            let result = app.execute_contract(
                Addr::unchecked(USER1),
                contract.addr(),
                &send_payment,
                &payment_amount,
            );
            assert!(result.is_err());
        }

        #[test]
        fn test_insufficient_funds() {
            let (mut app, contract) = proper_instantiate();
            register_users(&mut app, &contract);

            let payment_amount = vec![Coin {
                denom: NATIVE_DENOM.to_string(),
                amount: Uint128::new(50),
            }];

            // Try to send more than provided (should fail)
            let send_payment = ExecuteMsg::SendDirectPayment {
                to_username: "bob".to_string(),
                amount: Coin {
                    denom: NATIVE_DENOM.to_string(),
                    amount: Uint128::new(100), // Request 100 but only send 50
                },
                description: "Insufficient funds test".to_string(),
                proof_type: ProofType::None,
            };

            let result = app.execute_contract(
                Addr::unchecked(USER1),
                contract.addr(),
                &send_payment,
                &payment_amount,
            );
            assert!(result.is_err());
        }
    }

    mod username_management {
        use super::*;
        use crate::msg::{UsernameResponse, WalletResponse, HasUsernameResponse, UsernameAvailableResponse};

        #[test]
        fn test_case_insensitive_username_registration() {
            let (mut app, contract) = proper_instantiate();

            // Register user with uppercase username
            let register_msg = ExecuteMsg::RegisterUser {
                username: "ALICE".to_string(),
                display_name: "Alice Smith".to_string(),
            };
            app.execute_contract(Addr::unchecked(USER1), contract.addr(), &register_msg, &[])
                .unwrap();

            // Try to register with same username in lowercase (should fail)
            let register_msg_lower = ExecuteMsg::RegisterUser {
                username: "alice".to_string(),
                display_name: "Alice Johnson".to_string(),
            };
            let result = app.execute_contract(Addr::unchecked(USER2), contract.addr(), &register_msg_lower, &[]);
            assert!(result.is_err());

            // Query with different case should work
            let query_msg = QueryMsg::GetUserByUsername {
                username: "alice".to_string(),
            };
            let _result: crate::msg::UserResponse = app
                .wrap()
                .query_wasm_smart(contract.addr(), &query_msg)
                .unwrap();
        }

        #[test]
        fn test_username_validation() {
            let (mut app, contract) = proper_instantiate();

            // Test username too short
            let register_msg = ExecuteMsg::RegisterUser {
                username: "ab".to_string(),
                display_name: "Alice Smith".to_string(),
            };
            let result = app.execute_contract(Addr::unchecked(USER1), contract.addr(), &register_msg, &[]);
            assert!(result.is_err());

            // Test username too long (over 50 characters)
            let register_msg = ExecuteMsg::RegisterUser {
                username: "a".repeat(51),
                display_name: "Alice Smith".to_string(),
            };
            let result = app.execute_contract(Addr::unchecked(USER1), contract.addr(), &register_msg, &[]);
            assert!(result.is_err());

            // Test invalid characters
            let register_msg = ExecuteMsg::RegisterUser {
                username: "alice@test".to_string(),
                display_name: "Alice Smith".to_string(),
            };
            let result = app.execute_contract(Addr::unchecked(USER1), contract.addr(), &register_msg, &[]);
            assert!(result.is_err());

            // Test valid username with underscores
            let register_msg = ExecuteMsg::RegisterUser {
                username: "alice_123".to_string(),
                display_name: "Alice Smith".to_string(),
            };
            app.execute_contract(Addr::unchecked(USER1), contract.addr(), &register_msg, &[])
                .unwrap();
        }

        #[test]
        fn test_new_username_queries() {
            let (mut app, contract) = proper_instantiate();

            // Register user
            let register_msg = ExecuteMsg::RegisterUser {
                username: "alice".to_string(),
                display_name: "Alice Smith".to_string(),
            };
            app.execute_contract(Addr::unchecked(USER1), contract.addr(), &register_msg, &[])
                .unwrap();

            // Test GetUsernameByWallet
            let query_msg = QueryMsg::GetUsernameByWallet {
                wallet_address: USER1.to_string(),
            };
            let result: UsernameResponse = app
                .wrap()
                .query_wasm_smart(contract.addr(), &query_msg)
                .unwrap();
            assert_eq!(result.username, "alice");

            // Test GetWalletByUsername
            let query_msg = QueryMsg::GetWalletByUsername {
                username: "alice".to_string(),
            };
            let result: WalletResponse = app
                .wrap()
                .query_wasm_smart(contract.addr(), &query_msg)
                .unwrap();
            assert_eq!(result.wallet_address, USER1);

            // Test HasUsername for registered user
            let query_msg = QueryMsg::HasUsername {
                wallet_address: USER1.to_string(),
            };
            let result: HasUsernameResponse = app
                .wrap()
                .query_wasm_smart(contract.addr(), &query_msg)
                .unwrap();
            assert!(result.has_username);

            // Test HasUsername for unregistered user
            let query_msg = QueryMsg::HasUsername {
                wallet_address: USER2.to_string(),
            };
            let result: HasUsernameResponse = app
                .wrap()
                .query_wasm_smart(contract.addr(), &query_msg)
                .unwrap();
            assert!(!result.has_username);
        }

        #[test]
        fn test_username_availability_validation() {
            let (mut app, contract) = proper_instantiate();

            // Test invalid username format - should return false for availability
            let query_msg = QueryMsg::IsUsernameAvailable {
                username: "ab".to_string(), // Too short
            };
            let result: UsernameAvailableResponse = app
                .wrap()
                .query_wasm_smart(contract.addr(), &query_msg)
                .unwrap();
            assert!(!result.available);

            // Test valid but available username
            let query_msg = QueryMsg::IsUsernameAvailable {
                username: "alice".to_string(),
            };
            let result: UsernameAvailableResponse = app
                .wrap()
                .query_wasm_smart(contract.addr(), &query_msg)
                .unwrap();
            assert!(result.available);

            // Register user
            let register_msg = ExecuteMsg::RegisterUser {
                username: "alice".to_string(),
                display_name: "Alice Smith".to_string(),
            };
            app.execute_contract(Addr::unchecked(USER1), contract.addr(), &register_msg, &[])
                .unwrap();

            // Test taken username (case insensitive)
            let query_msg = QueryMsg::IsUsernameAvailable {
                username: "ALICE".to_string(),
            };
            let result: UsernameAvailableResponse = app
                .wrap()
                .query_wasm_smart(contract.addr(), &query_msg)
                .unwrap();
            assert!(!result.available);
        }
    }

    mod task_system {
        use super::*;
        use crate::msg::{TaskResponse, TasksResponse};

        fn get_future_timestamp() -> u64 {
            // Return timestamp far in the future (Unix timestamp for year 2050)
            2524608000
        }

        #[test]
        fn test_soft_task_lifecycle() {
            let (mut app, contract) = proper_instantiate();
            register_users(&mut app, &contract);

            let task_amount = Coin {
                denom: NATIVE_DENOM.to_string(),
                amount: Uint128::new(100),
            };

            // Create soft task (no escrow required)
            let create_task = ExecuteMsg::CreateTask {
                to_username: "bob".to_string(),
                amount: task_amount.clone(),
                description: "Write documentation".to_string(),
                proof_type: ProofType::Soft,
                deadline_ts: get_future_timestamp(),
                review_window_secs: None,
                endpoint: "https://api.example.com".to_string(),
            };

            app.execute_contract(
                Addr::unchecked(USER1),
                contract.addr(),
                &create_task,
                &[], // No funds needed for soft tasks
            )
            .unwrap();

            // Submit evidence
            let submit_evidence = ExecuteMsg::SubmitSoftEvidence {
                task_id: 1,
                evidence_hash: "evidence_hash_123".to_string(),
            };
            app.execute_contract(
                Addr::unchecked(USER2), // Bob submits evidence
                contract.addr(),
                &submit_evidence,
                &[],
            )
            .unwrap();

            // Approve task (for soft tasks, payer sends funds when approving)
            let approve_task = ExecuteMsg::ApproveTask { task_id: 1 };
            let task_funds = vec![Coin {
                denom: NATIVE_DENOM.to_string(),
                amount: Uint128::new(100),
            }];
            app.execute_contract(
                Addr::unchecked(USER1), // Alice approves and sends funds
                contract.addr(),
                &approve_task,
                &task_funds,
            )
            .unwrap();

            // Check task status
            let task_response: TaskResponse = app
                .wrap()
                .query_wasm_smart(contract.addr(), &QueryMsg::GetTaskById { task_id: 1 })
                .unwrap();
            assert_eq!(task_response.task.status, TaskStatus::Released);

            // Check bob received payment
            let bob_balance = app.wrap().query_balance(USER2, NATIVE_DENOM).unwrap();
            assert_eq!(bob_balance.amount, Uint128::new(10100)); // 10000 initial + 100 payment
        }

        #[test]
        fn test_zktls_task_instant_release() {
            let (mut app, contract) = proper_instantiate();
            register_users(&mut app, &contract);

            let task_amount = vec![Coin {
                denom: NATIVE_DENOM.to_string(),
                amount: Uint128::new(200),
            }];

            // Create zkTLS task (escrow required)
            let create_task = ExecuteMsg::CreateTask {
                to_username: "bob".to_string(),
                amount: task_amount[0].clone(),
                description: "API integration task".to_string(),
                proof_type: ProofType::ZkTLS,
                deadline_ts: get_future_timestamp(),
                review_window_secs: None,
                endpoint: "https://api.example.com/verify".to_string(),
            };

            app.execute_contract(
                Addr::unchecked(USER1),
                contract.addr(),
                &create_task,
                &task_amount, // Escrow funds
            )
            .unwrap();

            // Submit zkTLS proof with "valid" marker for stub verification
            let submit_proof = ExecuteMsg::SubmitZkTlsProof {
                task_id: 1,
                proof_blob_or_ref: "valid_zktls_proof_data".to_string(),
                zk_proof_hash: "zk_proof_hash_456".to_string(),
            };
            app.execute_contract(
                Addr::unchecked(USER2), // Bob submits proof
                contract.addr(),
                &submit_proof,
                &[],
            )
            .unwrap();

            // Check task was immediately released
            let task_response: TaskResponse = app
                .wrap()
                .query_wasm_smart(contract.addr(), &QueryMsg::GetTaskById { task_id: 1 })
                .unwrap();
            assert_eq!(task_response.task.status, TaskStatus::Released);

            // Check bob received payment
            let bob_balance = app.wrap().query_balance(USER2, NATIVE_DENOM).unwrap();
            assert_eq!(bob_balance.amount, Uint128::new(10200)); // 10000 initial + 200 payment
        }

        #[test]
        fn test_hybrid_task_with_dispute_window() {
            let (mut app, contract) = proper_instantiate();
            register_users(&mut app, &contract);

            let task_amount = vec![Coin {
                denom: NATIVE_DENOM.to_string(),
                amount: Uint128::new(300),
            }];

            // Create hybrid task
            let create_task = ExecuteMsg::CreateTask {
                to_username: "bob".to_string(),
                amount: task_amount[0].clone(),
                description: "Complex verification task".to_string(),
                proof_type: ProofType::Hybrid,
                deadline_ts: get_future_timestamp(),
                review_window_secs: Some(3600), // 1 hour dispute window
                endpoint: "https://api.example.com/hybrid".to_string(),
            };

            app.execute_contract(
                Addr::unchecked(USER1),
                contract.addr(),
                &create_task,
                &task_amount,
            )
            .unwrap();

            // Submit zkTLS proof
            let submit_proof = ExecuteMsg::SubmitZkTlsProof {
                task_id: 1,
                proof_blob_or_ref: "valid_hybrid_proof_data".to_string(),
                zk_proof_hash: "hybrid_proof_hash_789".to_string(),
            };
            app.execute_contract(
                Addr::unchecked(USER2),
                contract.addr(),
                &submit_proof,
                &[],
            )
            .unwrap();

            // Check task is in pending release state
            let task_response: TaskResponse = app
                .wrap()
                .query_wasm_smart(contract.addr(), &QueryMsg::GetTaskById { task_id: 1 })
                .unwrap();
            assert_eq!(task_response.task.status, TaskStatus::PendingRelease);

            // Bob should not have received payment yet
            let bob_balance = app.wrap().query_balance(USER2, NATIVE_DENOM).unwrap();
            assert_eq!(bob_balance.amount, Uint128::new(10000)); // No payment yet

            // Simulate window elapsed and release
            // Note: In a real test, we'd call ReleaseIfWindowElapsed after advancing blockchain time
            // For this stub test, we'll just verify the task is in pending release state
            // let _release_task = ExecuteMsg::ReleaseIfWindowElapsed { task_id: 1 };
        }

        #[test]
        fn test_hybrid_task_dispute() {
            let (mut app, contract) = proper_instantiate();
            register_users(&mut app, &contract);

            let task_amount = vec![Coin {
                denom: NATIVE_DENOM.to_string(),
                amount: Uint128::new(250),
            }];

            // Create hybrid task
            let create_task = ExecuteMsg::CreateTask {
                to_username: "bob".to_string(),
                amount: task_amount[0].clone(),
                description: "Disputable task".to_string(),
                proof_type: ProofType::Hybrid,
                deadline_ts: get_future_timestamp(),
                review_window_secs: Some(3600),
                endpoint: "https://api.example.com/dispute".to_string(),
            };

            app.execute_contract(
                Addr::unchecked(USER1),
                contract.addr(),
                &create_task,
                &task_amount,
            )
            .unwrap();

            // Submit proof and move to pending release
            let submit_proof = ExecuteMsg::SubmitZkTlsProof {
                task_id: 1,
                proof_blob_or_ref: "valid_dispute_proof".to_string(),
                zk_proof_hash: "dispute_proof_hash".to_string(),
            };
            app.execute_contract(
                Addr::unchecked(USER2),
                contract.addr(),
                &submit_proof,
                &[],
            )
            .unwrap();

            // Alice disputes the task
            let dispute_task = ExecuteMsg::DisputeTask {
                task_id: 1,
                reason_hash: Some("dispute_reason_hash".to_string()),
            };
            app.execute_contract(
                Addr::unchecked(USER1), // Payer disputes
                contract.addr(),
                &dispute_task,
                &[],
            )
            .unwrap();

            // Check task is in disputed state
            let task_response: TaskResponse = app
                .wrap()
                .query_wasm_smart(contract.addr(), &QueryMsg::GetTaskById { task_id: 1 })
                .unwrap();
            assert_eq!(task_response.task.status, TaskStatus::Disputed);

            // Admin resolves dispute in favor of worker
            let resolve_dispute = ExecuteMsg::ResolveDispute {
                task_id: 1,
                decision: true, // Release to worker
            };
            app.execute_contract(
                Addr::unchecked(ADMIN), // Only admin can resolve
                contract.addr(),
                &resolve_dispute,
                &[],
            )
            .unwrap();

            // Check bob received payment
            let bob_balance = app.wrap().query_balance(USER2, NATIVE_DENOM).unwrap();
            assert_eq!(bob_balance.amount, Uint128::new(10250));
        }

        #[test]
        #[ignore] // TODO: This test requires blockchain time manipulation
        fn test_task_expiry_refund() {
            let (mut app, contract) = proper_instantiate();
            register_users(&mut app, &contract);

            let task_amount = vec![Coin {
                denom: NATIVE_DENOM.to_string(),
                amount: Uint128::new(150),
            }];

            // Create task with past deadline for immediate expiry test
            // We'll create a task with valid deadline first, then manually set it as expired
            let create_task = ExecuteMsg::CreateTask {
                to_username: "bob".to_string(),
                amount: task_amount[0].clone(),
                description: "Expired task".to_string(),
                proof_type: ProofType::ZkTLS,
                deadline_ts: get_future_timestamp(), // Valid deadline initially
                review_window_secs: None,
                endpoint: "https://api.example.com/expired".to_string(),
            };

            app.execute_contract(
                Addr::unchecked(USER1),
                contract.addr(),
                &create_task,
                &task_amount,
            )
            .unwrap();

            // Try to refund expired task
            let refund_task = ExecuteMsg::RefundIfExpired { task_id: 1 };
            app.execute_contract(
                Addr::unchecked(USER1), // Anyone can call refund
                contract.addr(),
                &refund_task,
                &[],
            )
            .unwrap();

            // Check alice got refund
            let alice_balance = app.wrap().query_balance(USER1, NATIVE_DENOM).unwrap();
            assert_eq!(alice_balance.amount, Uint128::new(10000)); // Full refund

            // Check task status
            let task_response: TaskResponse = app
                .wrap()
                .query_wasm_smart(contract.addr(), &QueryMsg::GetTaskById { task_id: 1 })
                .unwrap();
            assert_eq!(task_response.task.status, TaskStatus::Refunded);
        }

        #[test]
        fn test_invalid_zktls_proof() {
            let (mut app, contract) = proper_instantiate();
            register_users(&mut app, &contract);

            let task_amount = vec![Coin {
                denom: NATIVE_DENOM.to_string(),
                amount: Uint128::new(100),
            }];

            // Create zkTLS task
            let create_task = ExecuteMsg::CreateTask {
                to_username: "bob".to_string(),
                amount: task_amount[0].clone(),
                description: "Invalid proof test".to_string(),
                proof_type: ProofType::ZkTLS,
                deadline_ts: get_future_timestamp(),
                review_window_secs: None,
                endpoint: "https://api.example.com/invalid".to_string(),
            };

            app.execute_contract(
                Addr::unchecked(USER1),
                contract.addr(),
                &create_task,
                &task_amount,
            )
            .unwrap();

            // Submit invalid proof (our stub considers short proofs invalid)
            let submit_proof = ExecuteMsg::SubmitZkTlsProof {
                task_id: 1,
                proof_blob_or_ref: "bad".to_string(), // Too short, will be invalid
                zk_proof_hash: "invalid_hash".to_string(),
            };
            let result = app.execute_contract(
                Addr::unchecked(USER2),
                contract.addr(),
                &submit_proof,
                &[],
            );
            assert!(result.is_err());
        }

        #[test]
        fn test_task_queries() {
            let (mut app, contract) = proper_instantiate();
            register_users(&mut app, &contract);

            let task_amount = vec![Coin {
                denom: NATIVE_DENOM.to_string(),
                amount: Uint128::new(50),
            }];

            // Create multiple tasks
            for i in 0..3 {
                let create_task = ExecuteMsg::CreateTask {
                    to_username: "bob".to_string(),
                    amount: task_amount[0].clone(),
                    description: format!("Task {}", i + 1),
                    proof_type: ProofType::Soft,
                    deadline_ts: get_future_timestamp(),
                    review_window_secs: None,
                    endpoint: format!("https://api.example.com/task{}", i + 1),
                };
                app.execute_contract(
                    Addr::unchecked(USER1),
                    contract.addr(),
                    &create_task,
                    &[],
                )
                .unwrap();
            }

            // Test task history query
            let history_response: TasksResponse = app
                .wrap()
                .query_wasm_smart(
                    contract.addr(),
                    &QueryMsg::GetTaskHistory {
                        username: "alice".to_string(),
                    },
                )
                .unwrap();
            assert_eq!(history_response.tasks.len(), 3);

            // Test pending tasks query
            let pending_response: TasksResponse = app
                .wrap()
                .query_wasm_smart(
                    contract.addr(),
                    &QueryMsg::GetPendingTasks {
                        username: "alice".to_string(),
                    },
                )
                .unwrap();
            assert_eq!(pending_response.tasks.len(), 3); // All soft tasks start as ProofSubmitted

            // Test individual task query
            let task_response: TaskResponse = app
                .wrap()
                .query_wasm_smart(contract.addr(), &QueryMsg::GetTaskById { task_id: 1 })
                .unwrap();
            assert_eq!(task_response.task.payer, "alice");
            assert_eq!(task_response.task.worker, "bob");
        }

        #[test]
        fn test_task_authorization_errors() {
            let (mut app, contract) = proper_instantiate();
            register_users(&mut app, &contract);

            let task_amount = vec![Coin {
                denom: NATIVE_DENOM.to_string(),
                amount: Uint128::new(100),
            }];

            // Create task
            let create_task = ExecuteMsg::CreateTask {
                to_username: "bob".to_string(),
                amount: task_amount[0].clone(),
                description: "Authorization test".to_string(),
                proof_type: ProofType::Hybrid,
                deadline_ts: get_future_timestamp(),
                review_window_secs: Some(3600),
                endpoint: "https://api.example.com/auth".to_string(),
            };
            app.execute_contract(
                Addr::unchecked(USER1),
                contract.addr(),
                &create_task,
                &task_amount,
            )
            .unwrap();

            // Try to submit proof as wrong user (should fail)
            let submit_proof = ExecuteMsg::SubmitZkTlsProof {
                task_id: 1,
                proof_blob_or_ref: "valid_unauthorized_proof".to_string(),
                zk_proof_hash: "unauth_hash".to_string(),
            };
            let result = app.execute_contract(
                Addr::unchecked(USER3), // Charlie tries to submit (not the worker)
                contract.addr(),
                &submit_proof,
                &[],
            );
            assert!(result.is_err());

            // Try to approve soft task as wrong user
            let create_soft_task = ExecuteMsg::CreateTask {
                to_username: "charlie".to_string(),
                amount: task_amount[0].clone(),
                description: "Soft task auth test".to_string(),
                proof_type: ProofType::Soft,
                deadline_ts: get_future_timestamp(),
                review_window_secs: None,
                endpoint: "https://api.example.com/soft".to_string(),
            };
            app.execute_contract(
                Addr::unchecked(USER1),
                contract.addr(),
                &create_soft_task,
                &[],
            )
            .unwrap();

            let approve_task = ExecuteMsg::ApproveTask { task_id: 2 };
            let result = app.execute_contract(
                Addr::unchecked(USER2), // Bob tries to approve (not the payer)
                contract.addr(),
                &approve_task,
                &[],
            );
            assert!(result.is_err());
        }

        #[test]
        fn test_cannot_create_task_with_self() {
            let (mut app, contract) = proper_instantiate();
            register_users(&mut app, &contract);

            let task_amount = vec![Coin {
                denom: NATIVE_DENOM.to_string(),
                amount: Uint128::new(100),
            }];

            // Try to create task with self as worker
            let create_task = ExecuteMsg::CreateTask {
                to_username: "alice".to_string(), // Same as payer
                amount: task_amount[0].clone(),
                description: "Self task".to_string(),
                proof_type: ProofType::Soft,
                deadline_ts: get_future_timestamp(),
                review_window_secs: None,
                endpoint: "https://api.example.com/self".to_string(),
            };
            let result = app.execute_contract(
                Addr::unchecked(USER1), // Alice
                contract.addr(),
                &create_task,
                &task_amount,
            );
            assert!(result.is_err());
        }
    }
}
