#[cfg(test)]
mod tests {
    use crate::helpers::SocialPaymentContract;
    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
    use crate::state::{PaymentStatus, ProofType};
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

            // Create help request with photo proof required
            let help_request = ExecuteMsg::CreateHelpRequest {
                to_username: "bob".to_string(),
                amount: payment_amount[0].clone(),
                description: "Help with moving".to_string(),
                proof_type: ProofType::Photo,
            };

            app.execute_contract(
                Addr::unchecked(USER1),
                contract.addr(),
                &help_request,
                &payment_amount,
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

            // Approve payment
            let approve_payment = ExecuteMsg::ApprovePayment { payment_id: 1 };
            app.execute_contract(
                Addr::unchecked(USER1),
                contract.addr(),
                &approve_payment,
                &[],
            )
            .unwrap();

            // Check bob received payment
            let bob_balance = app.wrap().query_balance(USER2, NATIVE_DENOM).unwrap();
            assert_eq!(bob_balance.amount, Uint128::new(10200)); // 10000 initial + 200 payment

            // Check payment status
            let payment_response: crate::msg::PaymentResponse = app
                .wrap()
                .query_wasm_smart(contract.addr(), &QueryMsg::GetPaymentById { payment_id: 1 })
                .unwrap();
            assert_eq!(payment_response.payment.status, PaymentStatus::Completed);
        }

        #[test]
        fn test_payment_cancellation() {
            let (mut app, contract) = proper_instantiate();
            register_users(&mut app, &contract);

            let payment_amount = vec![Coin {
                denom: NATIVE_DENOM.to_string(),
                amount: Uint128::new(150),
            }];

            // Create help request
            let help_request = ExecuteMsg::CreateHelpRequest {
                to_username: "bob".to_string(),
                amount: payment_amount[0].clone(),
                description: "Help with coding".to_string(),
                proof_type: ProofType::Manual,
            };

            app.execute_contract(
                Addr::unchecked(USER1),
                contract.addr(),
                &help_request,
                &payment_amount,
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

            // Check alice got refund
            let alice_balance = app.wrap().query_balance(USER1, NATIVE_DENOM).unwrap();
            assert_eq!(alice_balance.amount, Uint128::new(10000)); // Full refund

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
}
